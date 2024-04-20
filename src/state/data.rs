use crate::state::Variant;
use std::collections::HashMap;
use std::fmt::Debug;
use serde_json::{Value as SerdeValue, Number as SerdeNumber};
use serde::{Serialize, Deserialize};
use serde::ser::SerializeMap;
use mysql::Value as MysqlValue;

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub fields: HashMap<String, Variant>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub records: Vec<Record>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub tables: Vec<Table>,
    pub plan: Vec<(String,SerdeValue)>,
}

impl State {
    pub fn new() -> Self {
        State {
            tables: Vec::new(),
            plan: Vec::new(),
        }
    }
    pub fn find_table(&self, table_name: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.name == table_name)
    }
    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let state = serde_json::from_str(&json)?;
        Ok(state)
    }
}
