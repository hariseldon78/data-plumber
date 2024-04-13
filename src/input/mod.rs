use crate::state::{Record, Table};
use std::collections::HashMap;
use std::fs::read_to_string;
use crate::report;
use serde_json::{Value,Result};

pub fn read_file(path: String) -> Result<Table> {
    let input_file = read_to_string(path).unwrap();
    println!("{}",&input_file);
    let v: Value = serde_json::from_str(input_file.as_str())?;

    // Create fields and records from the parsed JSON
    let mut records: Vec<Record> = vec![];
    for record in v.as_array().unwrap() {
        let mut fields=HashMap::new();
        let map=record.as_object().unwrap();
        for entry in map {
            // FIXME: unnecessary clone, i'd like to move the ownership
            fields.insert(entry.0.clone(),entry.1.clone());
        }
        records.push(Record { fields});
    };

    // Construct a Table
    let table = Table {
        name: "ExampleTable".to_string(),
        records,
    };

    Ok(table)
}
