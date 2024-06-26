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

fn register_nodes(factory: &mut Factory) {
    InputMysql::register(factory);
    InputJson::register(factory);
    OutputSqlInserts::register(factory);
    OutputCompare::register(factory);
    OutputJson::register(factory);
    OutputRabbitMQ::register(factory);
}

fn main() -> Result<(), &'static str> {
    let config_file_name = std::env::args().nth(1);
    if config_file_name.is_none() {
        return Err("Usage: data-plumber config.json");
    }
    let rdr = std::fs::File::open(config_file_name.unwrap()).unwrap();
    let pipeline: Value = serde_json::from_reader(rdr).unwrap();
    let config = Config::from(&pipeline);
    let mut factory = Factory::new(&config).unwrap();
    register_nodes(&mut factory);

    let mut state = State::load_or_make(&config, &pipeline);

    for (key, value) in state.plan.clone().iter() {
        println!("Running node {}", key);
        let node = factory.create_node(key.clone(), &pipeline[key]);
        match node {
            Some(node) => {
                node.run(&mut state);
            }
            None => {
                continue;
            }
        };
        state.plan.remove(0);
        state.save("state.json").unwrap();
    }
    Ok(())
}
