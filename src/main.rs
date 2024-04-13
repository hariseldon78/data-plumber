#![allow(unused_variables)]
use std::fmt::Debug;

mod state;
mod input;
mod output;

use crate::input::read_file;
use crate::output::write_sql;

fn main() {
    let table=read_file(String::from("tests/input.json")).unwrap();
    // report(&table);
    let res=write_sql(&table).unwrap();
    println!("{}",&res);
}

fn report<T: Debug>(x: &T) {
    println!("{:#?}", x);
}
