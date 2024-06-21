use crate::state::{read_config_field, Factory, Process, Record, State, Table, Variant};
use crate::register_process;
use mysql::prelude::*;
use mysql::*;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::result::Result;

pub struct InputMysql {
    node_name: String,
    url: String,
    query: String,
}

impl Process for InputMysql {
    register_process!(input::mysql);
    fn from_config(node_name: String, config: Map<String, Value>) -> Self {
        InputMysql {
            node_name,
            url: read_config_field(&config, "url"),
            query: read_config_field(&config, "query"),
        }
    }
    fn run(&self, state: &mut State) {
        // println!("read_mysql_query({},{})", self.url, self.query);
        let pool = Pool::new(self.url.as_str()).unwrap();
        let result = self.query.clone().run(&pool).unwrap();
        let mut records: Vec<Record> = vec![];
        for row in result {
            let mut fields = HashMap::new();
            let map = row.unwrap();
            for column in map.columns_ref() {
                let value = &map[column.name_str().as_ref()];
                // println!("{}: {:?}", column.name_str(), value);
                fields.insert(
                    column.name_str().to_string(),
                    Variant::from_mysql_value(value.clone()),
                );
            }
            // println!("{:#?}", fields);
            records.push(Record { fields })
        }
        state.tables.push(Table {
            name: self.node_name.clone(),
            records,
        })
    }
}
