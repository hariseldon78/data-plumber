use crate::state::{Record, Table, Variant};
use mysql::prelude::*;
use mysql::*;
use serde_json::Value;
use std::collections::HashMap;
use std::result::Result;

pub struct InputMysql {
    pub url: String,
    pub query: String,
}


pub trait Configurable {
    fn from_config(config: &Value) -> Self;
}
impl Configurable for InputMysql {
    fn from_config(config: &Value) -> InputMysql {
        InputMysql {
            url: String::from(config["url"].as_str().unwrap()),
            query: String::from(config["query"].as_str().unwrap()),
        }
    }
}

pub trait InputNode {
    fn run(&self) -> Result<Table, String>;
}
impl InputNode for InputMysql {
    fn run(&self) -> Result<Table, String> {
        println!("read_mysql_query({},{})", self.url, self.query);
        let pool = Pool::new(self.url.as_str()).unwrap();
        let result = self.query.clone().run(&pool).unwrap();
        let mut records: Vec<Record> = vec![];
        for row in result {
            let mut fields = HashMap::new();
            let map = row.unwrap();
            for column in map.columns_ref() {
                let value = &map[column.name_str().as_ref()];
                fields.insert(
                    column.name_str().to_string(),
                    Variant::from_mysql_value(value.clone()),
                );
            }
            println!("{:#?}", fields);
            records.push(Record { fields })
        }
        Ok(Table {
            name: "unknown_name".to_string(),
            records,
        })
    }
}
