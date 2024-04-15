use crate::state::State;
use crate::input::*;
use crate::output::*;
use serde_json::Value;

pub fn read_config_field(config: &Value,name:&str)->String{
    String::from(config[name].as_str().unwrap())
}


pub trait Process {
    fn from_config(node_name: String,config: &Value) -> Self where Self:Sized;
    fn run(&self,state:&mut State);
}

struct Factory; 
impl Factory {
    pub fn create_node(node_name:String, config: &Value) -> Box<dyn Process> {
        match config["driver"].as_str() {
            Some("input::mysql") => Box::new(InputMysql::from_config(node_name.clone(), config)),
            Some("output::sql-inserts") => Box::new(OutputSqlInserts::from_config(node_name, config)),
            Some("output::compare") => Box::new(OutputCompare::from_config(node_name, config)),
            _ => panic!("Unknown driver: {}", config["driver"]),
        }
    }
}
