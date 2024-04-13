#![allow(unused_variables)]
use std::collections::HashMap;
use std::fmt::Debug;
use serde_json::{Value};

#[derive(Debug)]
pub struct Record {
    pub fields: HashMap<String, Value>,
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
