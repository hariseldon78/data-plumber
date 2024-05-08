use crate::state::{read_config_field, Factory, Process, Record, State, Table, Variant};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::result::Result;

pub struct OutputJson {
    input: String,
    path: String,
}

impl Process for OutputJson {
    fn register(factory: &mut Factory) {
        factory.register_process("output::json".to_string(), |node_name, config| {
            Box::new(Self::from_config(node_name, config))
        })
    }
    fn from_config(node_name: String, config: Map<String, Value>) -> Self {
        OutputJson {
            input: read_config_field(&config, "input"),
            path: read_config_field(&config, "path"),
        }
    }
    fn run(&self, state: &mut State) {
        let table = state.find_table(&self.input).unwrap();
        let file = std::fs::File::create(&self.path).unwrap();
        let mut writer = std::io::BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &table.records).unwrap();
    }
}
