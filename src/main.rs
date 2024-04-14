#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
// use std::fmt::Debug;

mod input;
mod output;
mod state;

use crate::input::json::read_json_file;
use crate::input::mysql::read_mysql_query;
use crate::output::write_sql;

use serde_json::Value;

fn main() {
    // read configuration from config.json
    let rdr = std::fs::File::open("config.json").unwrap();
    let config: Value = serde_json::from_reader(rdr).unwrap();

    // let table=read_json_file(String::from("tests/input.json")).unwrap();
    let table = read_mysql_query(
        String::from(config["mysql_input"]["url"].as_str().unwrap()),
        String::from(config["mysql_input"]["query"].as_str().unwrap()),
    )
    .unwrap();
    // report(&table);
    let res = write_sql(&table).unwrap();
    println!("{}", &res);
}

// fn report<T: Debug>(x: &T) {
//     println!("{:#?}", x);
// }
