use crate::state::Table;
use std::result::Result;
fn join<T>(x: T) -> String
where
    T: Iterator,
    T::Item: ToString,
{
    return x.map(|key| key.to_string())
        .collect::<Vec<String>>()
        .join(", ");
}

pub fn write_sql(t: &Table) -> Result<String,String> {
    for record in &(t.records) {
        let fields_keys=join(record.fields.keys());
        let fields_values=join(record.fields.values());
        println!("insert into {} ({}) values ({});",t.name,fields_keys,fields_values);
    }
    Ok("ciao".to_string())
}
