use crate::state::Table;
use std::result::Result;

fn join<T>(x: T,joiner: &str) -> String
where
    T: Iterator,
    T::Item: ToString,
{
    return x.map(|key| key.to_string())
        .collect::<Vec<String>>()
        .join(joiner);
}

pub fn write_sql(t: &Table) -> Result<String,String> {
    let mut commands:Vec<String>=vec![];
    for record in &(t.records) {
        let fields_keys=join(record.fields.keys(),",");
        let fields_values=join(record.fields.values(),",");
        commands.push(format!("insert into {} ({}) values ({});",t.name,fields_keys,fields_values));
    }
    Ok(join(commands.iter(),"\n"))
}
