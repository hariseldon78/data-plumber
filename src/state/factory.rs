use crate::state::State;
use crate::input::*;
use crate::output::*;
use serde_json::Value;
use std::collections::HashMap;

pub fn read_config_field(config: &Value,name:&str)->String{
    String::from(config[name].as_str().unwrap())
}

pub trait Process {
    fn register(factory: &mut Factory) where Self:Sized;
    fn from_config(node_name: String,config: &Value) -> Self where Self:Sized;
    fn run(&self,state:&mut State);
}

pub struct Factory{
    registry: HashMap<String, Box<dyn Fn(String, &Value)->Box<dyn Process + 'static>>>,
}
impl Factory {
    pub fn new()->Self {
        Factory {
            registry: HashMap::new()
        }
    }

    pub fn register_process<F>(&mut self, name: String, constructor: F)
        where F: 'static + Fn(String, &Value) -> Box<dyn Process>
    {
        self.registry.insert(name, Box::new(constructor));
    }

    pub fn create_node(&self,node_name:String, config: &Value) -> Box<dyn Process + 'static> {
        let driver = config["driver"].as_str().expect("driver field must be a string");
        let constructor = self.registry.get(driver).expect("Unknown driver");
        return constructor(node_name,config);
    }
}
