#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
// use std::fmt::Debug;

mod input;
mod output;
mod state;

use crate::input::*;
use crate::output::*;
use crate::state::*;

use serde_json::Value;

fn main() {
    let config_file_name = std::env::args().nth(1).unwrap();
    let rdr = std::fs::File::open(config_file_name).unwrap();
    let config: Value = serde_json::from_reader(rdr).unwrap();
    let mut state = state::State { tables: Vec::new() };

    for (key, value) in config.as_object().unwrap() {
        println!("Running node {}", key);
        let node = Factory::create_node(key.clone(), &config[key]);
        node.run(&mut state);
    }
}
