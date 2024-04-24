use crate::state::{Config,Variant};
use mysql::Value as MysqlValue;
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Number as SerdeNumber, Value as SerdeValue};
use std::collections::HashMap;
use std::fmt::{Debug,Formatter};

#[derive(Debug)]
pub struct Record {
    pub fields: HashMap<String, Variant>,
}

impl Serialize for Record {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        if self.fields.is_empty() {
            return serializer.serialize_none();
        }
        let mut map = serializer.serialize_map(Some(self.fields.len()))?;
        for (key, value) in &self.fields {
            map.serialize_entry(key, &(value.to_serde_value()))?;
        }
        map.end()
    }
}
// Implementing the Deserialize trait for Record
impl<'de> Deserialize<'de> for Record {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RecordVisitor;

        impl<'de> Visitor<'de> for RecordVisitor {
            type Value = Record;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a map of String to Variant")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Record, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut fields = HashMap::new();
                while let Some((key, value)) = map.next_entry()? {
                    fields.insert(key, value);
                }
                Ok(Record { fields })
            }
        }

        deserializer.deserialize_map(RecordVisitor)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub records: Vec<Record>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub tables: Vec<Table>,
    pub plan: Vec<(String, SerdeValue)>,
}

impl State {
    pub fn new() -> Self {
        State {
            tables: Vec::new(),
            plan: Vec::new(),
        }
    }
    pub fn load_or_new(config: &Config, pipeline: &SerdeValue) -> State {
        let mut state;
        if std::fs::metadata(&config.state_file).is_ok() {
            state = State::load(&config.state_file).unwrap();
        } else {
            state = State::new();
            for (key, value) in pipeline.as_object().unwrap() {
                state.plan.push((key.clone(), value.clone()));
            }
        };
        state
    }
    pub fn find_table(&self, table_name: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.name == table_name)
    }
    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let state = serde_json::from_str(&json)?;
        Ok(state)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json; // Make sure to include serde_json in your dependencies for serialization

    #[test]
    fn test_serialize_json_string() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), Variant::String("Alice".to_string()));

        let record = Record { fields };

        // Serialize the Record
        let serialized = dbg!(serde_json::to_string(&record).expect("Failed to serialize"));

        assert_eq!(
            serialized,
            r#"{"name":"Alice"}"#
        );

    }

    #[test]
    fn test_serialize_json_int() {
        let mut fields = HashMap::new();
        fields.insert("age".to_string(), Variant::Int(30));

        let record = Record { fields };

        // Serialize the Record
        let serialized = dbg!(serde_json::to_string(&record).expect("Failed to serialize"));

        assert_eq!(
            serialized,
            r#"{"age":30}"#
        );
    }

    #[test]
    fn test_serialize_json_float() {
        let mut fields = HashMap::new();
        fields.insert("height".to_string(), Variant::Float(5.9));

        let record = Record { fields };

        // Serialize the Record
        let serialized = dbg!(serde_json::to_string(&record).expect("Failed to serialize"));

        assert_eq!(
            serialized,
            r#"{"height":5.9}"#
        );
    }

    #[test]
    fn test_serialize_deserialize_record() {
        // Create a sample Record to test
        let mut fields = HashMap::new();
        fields.insert("age".to_string(), Variant::Int(30));
        fields.insert("name".to_string(), Variant::String("Alice".to_string()));
        fields.insert("height".to_string(), Variant::Float(5.9));

        let record = Record { fields };

        // Serialize the Record
        let serialized = dbg!(serde_json::to_string(&record).expect("Failed to serialize"));
        println!("Serialized: {}", serialized);

        // Deserialize the Record
        let deserialized: Record = serde_json::from_str(&serialized).expect("Failed to deserialize");

        // Check if the deserialized Record matches the original
        assert_eq!(deserialized.fields.get("age"), Some(&Variant::Int(30)));
        assert_eq!(deserialized.fields.get("name"), Some(&Variant::String("Alice".to_string())));
        assert_eq!(deserialized.fields.get("height"), Some(&Variant::Float(5.9)));
    }
}
