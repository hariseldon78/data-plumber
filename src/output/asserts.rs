use crate::state::{read_config_field, Factory, Process, State, Table};
use itertools::Itertools;
use serde_json::{Map, Value};
use std::io::Write;
use std::{cmp::Ordering, fs::File, result::Result};

#[derive(Debug, PartialEq)]
pub enum AssertState {
    Empty,
    RowsCountLessThan(usize),
    RowsCountEqual(usize),
    HasField(String),
    Not(Box<AssertState>),
    Or(Vec<AssertState>),
    And(Vec<AssertState>),
}
impl AssertState {
    fn parse(value: &Value) -> Self {
        let (key, sub_value) = value.as_object().unwrap().iter().next().unwrap();
        match (key.as_str(), sub_value) {
            ("empty", _) => AssertState::Empty,
            ("has-field", field_name) => {
                AssertState::HasField(String::from(field_name.as_str().unwrap()))
            }
            ("rows-count-less-than", count) => {
                AssertState::RowsCountLessThan(count.as_u64().unwrap() as usize)
            }
            ("rows-count-equal", count) => {
                AssertState::RowsCountEqual(count.as_u64().unwrap() as usize)
            }
            ("or", states) => {
                let states = states
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(AssertState::parse)
                    .collect();
                AssertState::Or(states)
            }
            ("and", states) => {
                let states = states
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(AssertState::parse)
                    .collect();
                AssertState::And(states)
            }
            ("not", state) => AssertState::Not(Box::new(AssertState::parse(state))),
            //  ("or", states)=>
            _ => panic!("unexpected state"),
        }
    }

    fn check(&self, table: &Table) -> (bool, Vec<String>) {
        let (result, error_messages) = match self {
            AssertState::Empty => (
                table.records.is_empty(),
                vec!["table is not empty".to_string()],
            ),
            AssertState::RowsCountLessThan(count) => (
                table.records.len() < *count,
                vec!["table has more rows than expected".to_string()],
            ),
            AssertState::RowsCountEqual(count) => (
                table.records.len() == *count,
                vec!["table has different number of rows than expected".to_string()],
            ),
            AssertState::HasField(field_name) => {
                let has_field = table
                    .records
                    .iter()
                    .all(|record| record.fields.contains_key(field_name));
                if has_field {
                    (true, vec![])
                } else {
                    (
                        false,
                        vec![format!("table does not have field {}", field_name)],
                    )
                }
            }
            AssertState::Not(state) => {
                let (result, errors) = state.check(table);
                (!result, errors)
            }
            AssertState::Or(states) => {
                let (result, errors) = states.iter().map(|state| state.check(table)).fold(
                    (false, vec![]),
                    |(acc_result, acc_errors), result| {
                        (
                            acc_result || result.0,
                            acc_errors.into_iter().chain(result.1.into_iter()).collect(),
                        )
                    },
                );
                (result, errors)
            }
            AssertState::And(states) => {
                let (result, errors) = states.iter().map(|state| state.check(table)).fold(
                    (true, vec![]),
                    |(acc_result, acc_errors), result| {
                        (
                            acc_result && result.0,
                            acc_errors.into_iter().chain(result.1.into_iter()).collect(),
                        )
                    },
                );
                (result, errors)
            }
        };
        if result {
            (true, vec![])
        } else {
            (false, error_messages)
        }
    }
}

pub struct Assert {
    table: String,
    state: AssertState,
}

pub struct OutputAsserts {
    node_name: String,
    asserts: Vec<Assert>,
}

impl Process for OutputAsserts {
    fn register(factory: &mut Factory) {
        factory.register_process("output::asserts".to_string(), |node_name, config| {
            Box::new(Self::from_config(node_name, config))
        })
    }
    fn from_config(node_name: String, config: Map<String, Value>) -> Self {
        let asserts = config
            .get("asserts")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|assert| {
                let table = assert.get("table").unwrap().as_str().unwrap();
                let state = AssertState::parse(assert.get("state").unwrap());
                Assert {
                    table: String::from(table),
                    state,
                }
            })
            .collect();
        Self { node_name, asserts }
    }
    fn run(&self, state: &mut State) {
        /* we check each assert, and store all the errors in a vector, and at the end we write them in a "<node_name>_results.txt" file
         */

        let mut errors: Vec<String> = vec![];

        for assert in &self.asserts {
            let table = state.find_table(assert.table.as_str()).unwrap();
            let (result, ass_errors) = assert.state.check(table);
            if !result {
                errors.push(format!(
                    "table {} failed assert: {:?}",
                    assert.table, ass_errors
                ));
            }
        }

        let file_name = format!("{}_results.txt", self.node_name);
        let mut file = File::create(file_name).unwrap();
        for error in errors {
            file.write_all(error.as_bytes()).unwrap();
            file.write_all("\n".as_bytes()).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_deserialize_simple_state() {
        let json_string = r#"
{
  "state":{
    "empty":""
  }
}
"#;

        let parsed: Value = serde_json::from_str(json_string).unwrap();
        let state = AssertState::parse(parsed.get("state").unwrap());
        assert_eq!(state, AssertState::Empty);
    }

    #[test]
    fn test_deserialize_nested_state() {
        let json_string = r#"
{
  "state":{
    "not": {
      "empty":""
    }
  }
}
"#;

        let parsed: Value = serde_json::from_str(json_string).unwrap();
        let state = AssertState::parse(parsed.get("state").unwrap());

        match state {
            AssertState::Not(x) => assert_eq!(*x, AssertState::Empty),
            _ => panic!("wrong state"),
        }
    }

    #[test]
    fn test_deserialize_and_state() {
        let json_string = r#"
{
    "state":{
        "and": [
        {
            "empty":""
        },
        {
            "has-field":"name"
        }
        ]
    }
}"#;

        let parsed: Value = serde_json::from_str(json_string).unwrap();
        let state = AssertState::parse(parsed.get("state").unwrap());

        match state {
            AssertState::And(x) => {
                assert_eq!(x.len(), 2);
                assert_eq!(x[0], AssertState::Empty);
                match &x[1] {
                    AssertState::HasField(field) => assert_eq!(field, "name"),
                    _ => panic!("wrong state"),
                }
            }
            _ => panic!("wrong state"),
        }
    }

    #[test]
    fn test_deserialize_or_state() {
        let json_string = r#"
{
    "state":{
        "or": [
        {
            "empty":""
        },
        {
            "has-field":"name"
        }
        ]
    }
}"#;

        let parsed: Value = serde_json::from_str(json_string).unwrap();
        let state = AssertState::parse(parsed.get("state").unwrap());

        match state {
            AssertState::Or(x) => {
                assert_eq!(x.len(), 2);
                assert_eq!(x[0], AssertState::Empty);
                match &x[1] {
                    AssertState::HasField(field) => assert_eq!(field, "name"),
                    _ => panic!("wrong state"),
                }
            }
            _ => panic!("wrong state"),
        }
    }

    #[test]
    fn test_deserialize_nested_or_state() {
        let json_string = r#"
{
    "state":{
        "or": [
        {
            "empty":""
        },
        {
            "or": [
                {
                    "has-field":"name"
                },
                {
                    "rows-count-equal": 10
                }
            ]
        }
        ]
    }
}"#;

        let parsed: Value = serde_json::from_str(json_string).unwrap();
        let state = AssertState::parse(parsed.get("state").unwrap());

        match state {
            AssertState::Or(x) => {
                assert_eq!(x.len(), 2);
                assert_eq!(x[0], AssertState::Empty);
                match &x[1] {
                    AssertState::Or(y) => {
                        assert_eq!(y.len(), 2);
                        match &y[0] {
                            AssertState::HasField(field) => assert_eq!(field, "name"),
                            _ => panic!("wrong state"),
                        }
                        match &y[1] {
                            AssertState::RowsCountEqual(count) => assert_eq!(*count, 10),
                            _ => panic!("wrong state"),
                        }
                    }
                    _ => panic!("wrong state"),
                }
            }
            _ => panic!("wrong state"),
        }
    }
}
