#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
// use std::fmt::Debug;

mod input;
mod output;
mod state;

use crate::input::json::read_json_file;
use crate::input::mysql::*;
use crate::output::write_sql;

use serde_json::Value;

fn main() {
    // get config file name from first argument
    let config_file_name = std::env::args().nth(1).unwrap();
    // read configuration from config.json
    let rdr = std::fs::File::open(config_file_name).unwrap();
    let config: Value = serde_json::from_reader(rdr).unwrap();

    // let table=read_json_file(String::from("tests/input.json")).unwrap();
    let node1=InputMysql::from_config(&config["mysql_input"]);
    let table = node1.run().unwrap();
    // report(&table);
    let res = write_sql(&table).unwrap();
    println!("{}", &res);
}

// fn report<T: Debug>(x: &T) {
//     println!("{:#?}", x);
// }
