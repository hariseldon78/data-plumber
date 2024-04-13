#![allow(unused_variables)]
use serde_json::{Result, Value};
use std::collections::HashMap;
use std::fmt::Debug;

mod state;
use crate::state::{Table,Record};
// #[derive(Debug)]
// struct Field {
//     name: String,
//     value: String,
// }

fn read_file() -> Result<Table> {
    let input_file = r#"
[
  {
    "a": 1,
    "b": "x"
  },
  {
    "a": 3,
    "b": "y"
  }
]"#;
    report(&input_file);
    let v: Value = serde_json::from_str(input_file)?;

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

fn main() {
    report(&read_file().unwrap());
}

fn report<T: Debug>(x: &T) {
    println!("{:?}", x);
}
