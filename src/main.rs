#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
// use std::fmt::Debug;

mod input;
mod output;
mod state;

use crate::input::json::read_json_file;
use crate::input::mysql::*;
use crate::output::OutputSqlInserts;
use crate::state::Process;

use serde_json::Value;

fn main() {
    // get config file name from first argument
    let config_file_name = std::env::args().nth(1).unwrap();
    // read configuration from config.json
    let rdr = std::fs::File::open(config_file_name).unwrap();
    let config: Value = serde_json::from_reader(rdr).unwrap();

    let mut state = state::State { tables: Vec::new() };

    let node1 = InputMysql::from_config("input1".to_string(), &config["input1"]);
    node1.run(&mut state);

    let node2 = OutputSqlInserts::from_config("output1".to_string(), &config["output1"]);
    node2.run(&mut state);

    // let res = write_sql(&state).unwrap();
    // println!("{}", &res);
}

// fn report<T: Debug>(x: &T) {
//     println!("{:#?}", x);
// }
