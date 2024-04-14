use crate::state::{Record, Table, Variant};
use mysql::prelude::*;
use mysql::*;
use std::collections::HashMap;
use std::result::Result;
use serde_json::Value;

pub fn read_mysql_query(url: String, query: String) -> Result<Table, String> {
    println!("read_mysql_query({},{})",url,query);
    let pool = Pool::new(url.as_str()).unwrap();
    let result = query.run(&pool).unwrap();
    let mut records: Vec<Record> = vec![];
    for row in result {
        let mut fields = HashMap::new();
        let map = row.unwrap();
        for column in map.columns_ref() {
            let value = &map[column.name_str().as_ref()];
            fields.insert(column.name_str().to_string(), Variant::from_mysql_value(value.clone()));
        }
        println!("{:#?}", fields);
        records.push(Record { fields })
    }
    Ok(Table {
        name: "unknown_name".to_string(),
        records,
    })
}
