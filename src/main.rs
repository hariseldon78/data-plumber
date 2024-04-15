#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
// use std::fmt::Debug;

mod input;
mod output;
mod state;

use std::process::exit;

use crate::input::*;
use crate::output::*;
use crate::state::*;

use serde_json::Value;

fn main()->Result<(),&'static str> {
    let config_file_name = std::env::args().nth(1);
    if config_file_name == None {
        return Err("Usage: data-plumber config.json");
    }
    let rdr = std::fs::File::open(config_file_name.unwrap()).unwrap();
    let config: Value = serde_json::from_reader(rdr).unwrap();
    let mut state = state::State { tables: Vec::new() };
    let mut factory = Factory::new();
    InputMysql::register(&mut factory);
    OutputSqlInserts::register(&mut factory);
    OutputCompare::register(&mut factory);

    for (key, value) in config.as_object().unwrap() {
        println!("Running node {}", key);
        let node = factory.create_node(key.clone(), &config[key]);
        node.run(&mut state);
    }
    Ok(())
}
