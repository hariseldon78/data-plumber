use crate::state::{read_config_field, Factory, Process, Record, State, Table, Variant};
use crate::register_process;
use itertools::Itertools;
use serde_json::{Map, Value};
use std::collections::HashMap;
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

    fn check(&self, table: &Table) -> (bool, Vec<String>, Vec<String>) {
        let (result, error_messages, description) = match self {
            AssertState::Empty => (
                table.records.is_empty(),
                vec!["table is not empty".to_string()],
                vec!["to be empty".to_string()],
            ),
            AssertState::RowsCountLessThan(count) => (
                table.records.len() < *count,
                vec!["table has more rows than expected".to_string()],
                vec![format!("to have less than {} rows", count).to_string()],
            ),
            AssertState::RowsCountEqual(count) => (
                table.records.len() == *count,
                vec!["table has different number of rows than expected".to_string()],
                vec![format!("to have exactly {} rows", count).to_string()],
            ),
            AssertState::HasField(field_name) => {
                let description = format!("to have field {}", field_name);
                let has_field = table
                    .records
                    .iter()
                    .all(|record| record.fields.contains_key(field_name));
                if has_field {
                    (true, vec![], vec![description])
                } else {
                    (
                        false,
                        vec![format!("table does not have field {}", field_name)],
                        vec![description],
                    )
                }
            }
            AssertState::Not(state) => {
                let (result, errors, description) = state.check(table);
                let mut description2 = vec!["not".to_string()];
                for desc in description.iter() {
                    description2.push(desc.clone());
                }

                if result {
                    let error = format!("not ({})", description.join(", "));
                    (false, vec![error], description2)
                } else {
                    (true, vec![], description2)
                }
            }
            AssertState::Or(states) => {
                let (result, errors, description) =
                    states.iter().map(|state| state.check(table)).fold(
                        (false, vec![], vec!["or".to_string()]),
                        |(acc_result, acc_errors, acc_description), result| {
                            (
                                acc_result || result.0,
                                acc_errors.into_iter().chain(result.1).collect(),
                                acc_description.into_iter().chain(result.2).collect(),
                            )
                        },
                    );
                (result, errors, description)
            }
            AssertState::And(states) => {
                let (result, errors, description) =
                    states.iter().map(|state| state.check(table)).fold(
                        (true, vec![], vec!["and".to_string()]),
                        |(acc_result, acc_errors, acc_description), result| {
                            (
                                acc_result && result.0,
                                acc_errors.into_iter().chain(result.1).collect(),
                                acc_description.into_iter().chain(result.2).collect(),
                            )
                        },
                    );
                (result, errors, description)
            }
        };
        if result {
            (true, vec![], description)
        } else {
            (false, error_messages, description)
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
    register_process!(output::asserts);
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
            let (result, ass_errors, _) = assert.state.check(table);
            if !result {
                errors.push(format!(
                    "table {} failed assert: {:?}",
                    assert.table, ass_errors
                ));
            }
        }

        let file_name = format!("{}_results.txt", self.node_name);
        // let mut file = File::create(file_name).unwrap();
        // for error in errors {
        //     file.write_all(error.as_bytes()).unwrap();
        //     file.write_all("\n".as_bytes()).unwrap();
        // }
        state.write_file(file_name.as_str(), &errors);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Config, MemoryReader, MemoryWriter, Table};

    #[test]
    fn test_deserialize_simple_assert_state() {
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
    fn test_deserialize_nested_assert_state() {
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

    #[test]
    fn test_assert_empty() {
        let config = Config {
            state_file: "state.json".to_string(),
            template_file: "template.json".to_string(),
        };

        let pipeline = serde_json::from_str(
            r#"{
    "test": {
        "driver":"output::asserts",
        "asserts":[
            {
                "table":"table1",
                "state":{
                    "empty":""
                }
            }
        ]
    }
}"#,
        )
        .unwrap();
        let mut state = State::make(
            &config,
            &pipeline,
            Some(Box::new(MemoryWriter::new())),
            Some(Box::new(MemoryReader::new())),
        );

        state.tables.push(Table {
            name: "table1".to_string(),
            records: vec![],
        });
        let process = OutputAsserts::from_config(
            "test".to_string(),
            pipeline.get("test").unwrap().as_object().unwrap().clone(),
        );
        process.run(&mut state);

        assert_eq!(
            state
                .results_writer
                .test_peek("test_results.txt")
                .unwrap()
                .len(),
            0
        );
    }

    #[test]
    fn test_assert_empty_fail() {
        let config = Config {
            state_file: "state.json".to_string(),
            template_file: "template.json".to_string(),
        };

        let pipeline = serde_json::from_str(
            r#"{
    "test": {
        "driver":"output::asserts",
        "asserts":[
            {
                "table":"table1",
                "state":{
                    "empty":""
                }
            }
        ]
    }
}"#,
        )
        .unwrap();
        let mut state = State::make(
            &config,
            &pipeline,
            Some(Box::new(MemoryWriter::new())),
            Some(Box::new(MemoryReader::new())),
        );

        let mut rec1 = Record {
            fields: HashMap::new(),
        };
        rec1.fields
            .insert("name".to_string(), Variant::String("test".to_string()));
        state.tables.push(Table {
            name: "table1".to_string(),
            records: vec![rec1],
        });
        let process = OutputAsserts::from_config(
            "test".to_string(),
            pipeline.get("test").unwrap().as_object().unwrap().clone(),
        );
        process.run(&mut state);

        assert_eq!(
            state
                .results_writer
                .test_peek("test_results.txt")
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn test_assert_not_empty_and_less_than() {
        let config = Config {
            state_file: "state.json".to_string(),
            template_file: "template.json".to_string(),
        };

        let pipeline = serde_json::from_str(
            r#"{
    "test": {
        "driver":"output::asserts",
        "asserts":[
            {
                "table":"table1",
                "state":{
                    "and":[
                        {"not":{"empty":""}},
                        {"rows-count-less-than": 2}
                    ]
                }
            }
        ]
    }
}"#,
        )
        .unwrap();
        let mut state = State::make(
            &config,
            &pipeline,
            Some(Box::new(MemoryWriter::new())),
            Some(Box::new(MemoryReader::new())),
        );

        let mut rec1 = Record {
            fields: HashMap::new(),
        };
        rec1.fields
            .insert("name".to_string(), Variant::String("test".to_string()));
        state.tables.push(Table {
            name: "table1".to_string(),
            records: vec![rec1],
        });
        let process = OutputAsserts::from_config(
            "test".to_string(),
            pipeline.get("test").unwrap().as_object().unwrap().clone(),
        );
        process.run(&mut state);

        assert_eq!(
            state
                .results_writer
                .test_peek("test_results.txt")
                .unwrap()
                .len(),
            0
        );
    }

    #[test]
    fn test_assert_not_empty_and_less_than_fail1() {
        let config = Config {
            state_file: "state.json".to_string(),
            template_file: "template.json".to_string(),
        };

        let pipeline = serde_json::from_str(
            r#"{
    "test": {
        "driver":"output::asserts",
        "asserts":[
            {
                "table":"table1",
                "state":{
                    "and":[
                        {"not":{"empty":""}},
                        {"rows-count-less-than": 2}
                    ]
                }
            }
        ]
    }
}"#,
        )
        .unwrap();
        let mut state = State::make(
            &config,
            &pipeline,
            Some(Box::new(MemoryWriter::new())),
            Some(Box::new(MemoryReader::new())),
        );

        state.tables.push(Table {
            name: "table1".to_string(),
            records: vec![],
        });
        let process = OutputAsserts::from_config(
            "test".to_string(),
            pipeline.get("test").unwrap().as_object().unwrap().clone(),
        );
        process.run(&mut state);

        assert_eq!(
            state
                .results_writer
                .test_peek("test_results.txt")
                .unwrap()
                .len(),
            1
        );
    }
    #[test]
    fn test_assert_not_empty_and_less_than_fail2() {
        let config = Config {
            state_file: "state.json".to_string(),
            template_file: "template.json".to_string(),
        };

        let pipeline = serde_json::from_str(
            r#"{
    "test": {
        "driver":"output::asserts",
        "asserts":[
            {
                "table":"table1",
                "state":{
                    "and":[
                        {"not":{"empty":""}},
                        {"rows-count-less-than": 2}
                    ]
                }
            }
        ]
    }
}"#,
        )
        .unwrap();
        let mut state = State::make(
            &config,
            &pipeline,
            Some(Box::new(MemoryWriter::new())),
            Some(Box::new(MemoryReader::new())),
        );

        let mut rec1 = Record {
            fields: HashMap::new(),
        };
        rec1.fields
            .insert("name".to_string(), Variant::String("test".to_string()));
        let mut rec2 = Record {
            fields: HashMap::new(),
        };
        rec2.fields
            .insert("name".to_string(), Variant::String("test2".to_string()));
        state.tables.push(Table {
            name: "table1".to_string(),
            records: vec![rec1, rec2],
        });
        let process = OutputAsserts::from_config(
            "test".to_string(),
            pipeline.get("test").unwrap().as_object().unwrap().clone(),
        );
        process.run(&mut state);

        assert_eq!(
            state
                .results_writer
                .test_peek("test_results.txt")
                .unwrap()
                .len(),
            1
        );
    }
}
