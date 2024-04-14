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
#[derive(Debug)]
pub enum Variant {
    Null,
    String(String),
    Float(f64),
    Int(i64),
}

impl Variant {
    pub fn from_serde_value(value: &SerdeValue) -> Variant {
        match value {
            SerdeValue::Null => Variant::Null,
            SerdeValue::String(s) => Variant::String(s.clone()),
            SerdeValue::Number(n) => {
                if n.is_f64() {
                    Variant::Float(n.as_f64().unwrap())
                } else {
                    Variant::Int(n.as_i64().unwrap())
                }
            }
            _ => panic!("Unsupported value type"),
        }
    }

    pub fn from_mysql_value(value: MysqlValue) -> Variant {
        match value {
            MysqlValue::NULL => Variant::Null,
            MysqlValue::Bytes(bytes) => {
                let s = String::from_utf8(bytes).unwrap();
                Variant::String(s)
            }
            MysqlValue::Int(i) => Variant::Int(i),
            MysqlValue::UInt(u) => Variant::Int(u as i64),
            MysqlValue::Float(f) => Variant::Float(f as f64),
            MysqlValue::Double(d) => Variant::Float(d),
            _ => panic!("Unsupported value type"),
        }
    }

    pub fn to_mysql_value(&self) -> MysqlValue {
        match self {
            Variant::Null => MysqlValue::NULL,
            Variant::String(s) => MysqlValue::Bytes(s.as_bytes().to_vec()),
            Variant::Int(i) => MysqlValue::Int(*i),
            Variant::Float(f) => MysqlValue::Double(*f),
        }
    }

    pub fn to_serde_value(&self) -> SerdeValue {
        match self {
            Variant::Null => SerdeValue::Null,
            Variant::String(s) => SerdeValue::String(s.clone()),
            Variant::Int(i) => SerdeValue::Number((*i).into()),
            Variant::Float(f) => SerdeValue::Number(SerdeNumber::from_f64(*f).unwrap()),
        }
    }
}

impl std::fmt::Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let serde=self.to_serde_value();
        write!(f, "{}", serde)
    }
}
