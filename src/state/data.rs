use crate::state::Variant;
use std::collections::HashMap;
use std::fmt::Debug;
use serde_json::{Value as SerdeValue, Number as SerdeNumber};
use serde::{Serialize, Serializer};
use serde::ser::SerializeMap;
use mysql::Value as MysqlValue;

#[derive(Debug)]
pub struct Record {
    pub fields: HashMap<String, Variant>,
}
impl Serialize for Record {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.fields.len()))?;
        for (k, v) in &self.fields {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
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
