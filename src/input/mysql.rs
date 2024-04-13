use mysql::*;
use std::result::Result;

pub fn read_mysql_query(url: String, query: String) -> Result<Table, String> {
    let pool = Pool::new(url)?;
    Ok(Table {
        name: "not implemented",
        records: vec![],
    })
}
