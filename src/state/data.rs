use crate::state::Variant;
use std::collections::HashMap;
use std::fmt::Debug;
use serde_json::{Value as SerdeValue, Number as SerdeNumber};
use mysql::Value as MysqlValue;

#[derive(Debug)]
pub struct Record {
    pub fields: HashMap<String, Variant>,
}

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub records: Vec<Record>,
}

#[derive(Debug)]
pub struct State {
    pub tables: Vec<Table>,
}

impl State {
    pub fn find_table(&self, table_name: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.name == table_name)
    }
}
