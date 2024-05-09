use crate::state::{Config, Table, Variant};
use mysql::Value as MysqlValue;
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Number as SerdeNumber, Value as SerdeValue};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

pub trait ResultsWriter: Debug {
    fn write(&mut self, name: &str, rows: &[String]);
    fn test_peek(&self, name: &str) -> Option<&Vec<String>> {
        None
    }
}

#[derive(Debug)]
pub struct FileWriter {}
impl ResultsWriter for FileWriter {
    fn write(&mut self, name: &str, rows: &[String]) {
        let path = String::from(name);
        let content = rows.join("\n");
        std::fs::write(path, content).unwrap();
    }
}

#[derive(Debug)]
pub struct MemoryWriter {
    pub results: HashMap<String, Vec<String>>,
}
impl MemoryWriter {
    pub fn new() -> Self {
        MemoryWriter {
            results: HashMap::new(),
        }
    }
}
impl ResultsWriter for MemoryWriter {
    fn write(&mut self, name: &str, rows: &[String]) {
        self.results.insert(name.to_string(), rows.to_vec());
    }
    fn test_peek(&self, name: &str) -> Option<&Vec<String>> {
        self.results.get(name)
    }
}

pub trait InputReader: Debug {
    fn read(&self, name: &str) -> Vec<String>;
}

#[derive(Debug)]
pub struct FileReader {}
impl InputReader for FileReader {
    fn read(&self, name: &str) -> Vec<String> {
        let path = String::from(name);
        let content = std::fs::read_to_string(path).unwrap();
        content.lines().map(|s| s.to_string()).collect()
    }
}

#[derive(Debug)]
pub struct MemoryReader {
    pub files: HashMap<String, Vec<String>>,
}
impl MemoryReader {
    pub fn new() -> Self {
        MemoryReader {
            files: HashMap::new(),
        }
    }
}
impl InputReader for MemoryReader {
    fn read(&self, name: &str) -> Vec<String> {
        self.files.get(name).unwrap().to_vec()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub tables: Vec<Table>,
    pub plan: Vec<(String, SerdeValue)>,
    #[serde(skip, default = "make_file_writer")]
    pub results_writer: Box<dyn ResultsWriter>,
    #[serde(skip, default = "make_file_reader")]
    pub input_reader: Box<dyn InputReader>,
}

fn make_file_writer() -> Box<dyn ResultsWriter> {
    Box::new(FileWriter {})
}

fn make_file_reader() -> Box<dyn InputReader> {
    Box::new(FileReader {})
}

impl State {
    pub fn new(
        results_writer: Option<Box<dyn ResultsWriter>>,
        input_reader: Option<Box<dyn InputReader>>,
    ) -> Self {
        State {
            tables: Vec::new(),
            plan: Vec::new(),
            results_writer: results_writer.unwrap_or(Box::new(FileWriter {})),
            input_reader: input_reader.unwrap_or(Box::new(FileReader {})),
        }
    }
    pub fn load_or_make(config: &Config, pipeline: &SerdeValue) -> State {
        if std::fs::metadata(&config.state_file).is_ok() {
            State::load(&config.state_file, None).unwrap()
        } else {
            State::make(config, pipeline, None, None)
        }
    }
    pub fn make(
        config: &Config,
        pipeline: &SerdeValue,
        results_writer: Option<Box<dyn ResultsWriter>>,
        input_reader: Option<Box<dyn InputReader>>,
    ) -> State {
        let mut state = State::new(results_writer, input_reader);
        for (key, value) in pipeline.as_object().unwrap() {
            state.plan.push((key.clone(), value.clone()));
        }
        state
    }
    pub fn find_table(&self, table_name: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.name == table_name)
    }
    pub fn save(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        self.results_writer.write(path, &[json]);
        Ok(())
    }
    pub fn load(
        path: &str,
        input_reader: Option<Box<dyn InputReader>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let reader = input_reader.unwrap_or(Box::new(FileReader {}));
        let json = reader.read(path).join("\n");
        let state = serde_json::from_str(&json)?;
        Ok(state)
    }
    pub fn write_file(&mut self, name: &str, rows: &[String]) {
        self.results_writer.write(name, rows);
    }
    pub fn read_file(&self, name: &str) -> Vec<String> {
        self.input_reader.read(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Make sure to include serde_json in your dependencies for serialization

    #[test]
    fn test_state_creation() {
        let config = Config {
            state_file: "state.json".to_string(),
            template_file: "template.json".to_string(),
        };
        let pipeline = serde_json::from_str(r#"{"test":{"driver":"none"}}"#).unwrap();
        let state = State::make(
            &config,
            &pipeline,
            Some(Box::new(MemoryWriter::new())),
            Some(Box::new(MemoryReader::new())),
        );
    }
}
