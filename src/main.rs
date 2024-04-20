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

fn register_nodes(mut factory: &mut Factory) {
    InputMysql::register(&mut factory);
    InputJson::register(&mut factory);
    OutputSqlInserts::register(&mut factory);
    OutputCompare::register(&mut factory);
    OutputJson::register(&mut factory);
}

fn main() -> Result<(), &'static str> {
    let config_file_name = std::env::args().nth(1);
    if config_file_name == None {
        return Err("Usage: data-plumber config.json");
    }
    let mut factory = Factory::new();
    register_nodes(&mut factory);
    let rdr = std::fs::File::open(config_file_name.unwrap()).unwrap();
    let config: Value = serde_json::from_reader(rdr).unwrap();
    // if there is a file 'state.json' load the state, else init it
    let mut state;
    if std::fs::metadata("state.json").is_ok() {
        state = State::load("state.json").unwrap();

    } else {
        state = State::new();
        for (key, value) in config.as_object().unwrap() {
            state.plan.push((key.clone(), value.clone()));
        }
    };

    for (key, value) in state.plan.clone().iter() {
        println!("Running node {}", key);
        let node = factory.create_node(key.clone(), &config[key]);
        node.run(&mut state);
        println!("Saving state");
        state.plan.remove(0);
        state.save("state.json").unwrap();
    }
    Ok(())
}
