use crate::state::{Record, Table, State, Variant, read_config_field, Process, Factory};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::result::Result;

pub struct InputJson {
    node_name: String,
    path: String,
}

impl Process for InputJson {
    fn register(factory: &mut Factory){
        factory.register_process("input::json".to_string(), |node_name, config| {
            Box::new(Self::from_config(node_name, config))
        })
    }
    fn from_config(node_name: String, config: &Value) -> Self {
        InputJson {
            node_name,
            path: read_config_field(config,"path"),
        }
    }
    fn run(&self, state: &mut State) {
        let input_file = read_to_string(self.path.as_str()).unwrap();
        // println!("{}", &input_file);
        let v: Value = serde_json::from_str(input_file.as_str()).unwrap();

        // Create fields and records from the parsed JSON
        let mut records: Vec<Record> = vec![];
        for record in v.as_array().unwrap() {
            let mut fields = HashMap::new();
            let map = record.as_object().unwrap();
            for entry in map {
                // FIXME: unnecessary clone, i'd like to move the ownership
                fields.insert(entry.0.clone(), Variant::from_serde_value(&entry.1.clone()));
            }
            records.push(Record { fields });
        }

        // Construct a Table
        state.tables.push(Table {
            name: self.node_name.clone(),
            records,
        })
    }
}
