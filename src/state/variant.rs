use mysql::Value as MysqlValue;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Number as SerdeNumber, Value as SerdeValue};
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug)]
pub enum Variant {
    Null,
    String(String),
    Float(f64),
    Int(i64),
}

impl Variant {
    pub fn from_serde_value(value: &SerdeValue) -> Variant {
        match value {
            SerdeValue::Null => Variant::Null,
            SerdeValue::String(s) => Variant::String(s.clone()),
            SerdeValue::Number(n) => {
                if n.is_f64() {
                    Variant::Float(n.as_f64().unwrap())
                } else {
                    Variant::Int(n.as_i64().unwrap())
                }
            }
            _ => {
                println!("value: {:?}", value);
                panic!("Unsupported value type");
            }
        }
    }

    pub fn from_mysql_value(value: MysqlValue) -> Variant {
        match value {
            MysqlValue::NULL => Variant::Null,
            MysqlValue::Bytes(bytes) => {
                let s = String::from_utf8(bytes).unwrap();
                Variant::String(s)
            }
            MysqlValue::Int(i) => Variant::Int(i),
            MysqlValue::UInt(u) => Variant::Int(u as i64),
            MysqlValue::Float(f) => Variant::Float(f as f64),
            MysqlValue::Double(d) => Variant::Float(d),
            _ => panic!("Unsupported value type"),
        }
    }

    pub fn to_mysql_value(&self) -> MysqlValue {
        match self {
            Variant::Null => MysqlValue::NULL,
            Variant::String(s) => MysqlValue::Bytes(s.as_bytes().to_vec()),
            Variant::Int(i) => MysqlValue::Int(*i),
            Variant::Float(f) => MysqlValue::Double(*f),
        }
    }

    pub fn to_serde_value(&self) -> SerdeValue {
        match self {
            Variant::Null => SerdeValue::Null,
            Variant::String(s) => SerdeValue::String(s.clone()),
            Variant::Int(i) => SerdeValue::Number((*i).into()),
            Variant::Float(f) => SerdeValue::Number(SerdeNumber::from_f64(*f).unwrap()),
        }
    }
}

impl std::fmt::Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let serde = self.to_serde_value();
        write!(f, "{}", serde)
    }
}

// allow comparison of Variant
impl PartialEq for Variant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Variant::Null, Variant::Null) => true,
            (Variant::String(s1), Variant::String(s2)) => s1 == s2,
            (Variant::Int(i1), Variant::Int(i2)) => i1 == i2,
            (Variant::Float(f1), Variant::Float(f2)) => f1 == f2,
            _ => false,
        }
    }
}

impl Serialize for Variant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let serde = self.to_serde_value();
        serde.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Variant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let serde = SerdeValue::deserialize(deserializer)?;
        Ok(Variant::from_serde_value(&serde))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_string() {
        let v = Variant::String("hello".to_string());
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(s, "\"hello\"");
    }

    #[test]
    fn test_deserialize_string() {
        let s = "\"hello\"";
        let v: Variant = serde_json::from_str(s).unwrap();
        assert_eq!(v, Variant::String("hello".to_string()));
    }

    #[test]
    fn test_serialize_int() {
        let v = Variant::Int(123);
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(s, "123");
    }

    #[test]
    fn test_deserialize_int() {
        let s = "123";
        let v: Variant = serde_json::from_str(s).unwrap();
        assert_eq!(v, Variant::Int(123));
    }

    #[test]
    fn test_serialize_float() {
        let v = Variant::Float(123.45);
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(s, "123.45");
    }

    #[test]
    fn test_deserialize_float() {
        let s = "123.45";
        let v: Variant = serde_json::from_str(s).unwrap();
        assert_eq!(v, Variant::Float(123.45));
    }
}
