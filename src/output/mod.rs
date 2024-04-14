use crate::state::{Process, State, Table};
use serde_json::Value;
use std::{fs::File, result::Result};
use itertools::Itertools;

fn join<T>(x: T, joiner: Option<&str>) -> String
where
    T: Iterator,
    T::Item: ToString,
{
    return x
        .map(|key| key.to_string())
        .collect::<Vec<String>>()
        .join(joiner.unwrap_or(","));
}

pub struct OutputSqlInserts {
    node_name: String,
    input: String,
    path: String,
    table_name: String,
}

impl Process for OutputSqlInserts {
    fn from_config(node_name: String, config: &Value) -> Self {
        OutputSqlInserts {
            node_name,
            input: String::from(config["input"].as_str().unwrap()),
            path: String::from(config["path"].as_str().unwrap()),
            table_name: String::from(config["table-name"].as_str().unwrap()),
        }
    }
    fn run(&self, state: &mut State) {
        let t = state.find_table(self.input.as_str()).unwrap();

        let mut commands: Vec<String> = vec![];
        for record in &(t.records) {
            let sorted_keys = record.fields.keys().sorted();
            let fields_keys = join(sorted_keys.clone(), None);
            let fields_values = join(sorted_keys.map(|key| record.fields[key].to_string()), None);
            commands.push(format!(
                "insert into {} ({}) values ({});",
                self.table_name, fields_keys, fields_values
            ));
        }
        let output = join(commands.iter(), Some("\n"));

        std::fs::write(&self.path, output).unwrap();
    }
}
