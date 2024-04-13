#![allow(unused_variables)]
use std::fmt::Debug;

#[derive(Debug)]
struct Field {
    name: String,
    value: String,
}


#[derive(Debug)]
struct Record {
    fields: Vec<Field>,
}


#[derive(Debug)]
struct Table {
    name: String,
    records: Vec<Record>,
}

#[derive(Debug)]
struct State {
    tables: Vec<Table>,
}



fn main() {
    let f1 = Field {
        name: String::from("a"),
        value: String::from("1"),
    };
    let r1 = Record {
        fields: vec![f1]
    };
    let t = Table {
        name:String::from("Example"),
        records: vec![r1]
    };

    report(&t);
}

fn report<T:Debug>(x:&T){
    println!("{:?}",x);
}
