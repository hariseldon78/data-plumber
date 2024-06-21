use crate::input::*;
use crate::output::*;
use crate::state::{Config, State};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::io::Error;

pub fn read_config_field(config: &Map<String, Value>, name: &str) -> String {
    String::from(config[name].as_str().unwrap())
}

pub trait Process {
    fn register(factory: &mut Factory)
    where
        Self: Sized;
    fn from_config(node_name: String, config: Map<String, Value>) -> Self
    where
        Self: Sized;
    fn run(&self, state: &mut State);
}

#[macro_export]
macro_rules! register_process {
    ($name:expr) => {
        fn register(factory: &mut Factory) {
            factory.register_process(stringify!($name).to_string(), |node_name, config| {
                Box::new(Self::from_config(node_name, config))
            })
        }
    };
}

pub struct Factory {
    registry:
        HashMap<String, Box<dyn Fn(String, Map<String, Value>) -> Box<dyn Process + 'static>>>,
    templates: serde_json::Map<String, Value>,
}
impl Factory {
    pub fn new(config: &Config) -> Result<Self, Error> {
        let mut templates: serde_json::Map<String, Value> = serde_json::Map::new();

        if std::fs::metadata(&config.template_file).is_ok() {
            let rdr = std::fs::File::open(&config.template_file)?;
            let templates_content: Value = serde_json::from_reader(rdr)?;
            templates_content
                .as_object()
                .unwrap()
                .clone_into(&mut templates);
        }
        Ok(Factory {
            registry: HashMap::new(),
            templates,
        })
    }

    pub fn register_process<F>(&mut self, name: String, constructor: F)
    where
        F: 'static + Fn(String, Map<String, Value>) -> Box<dyn Process>,
    {
        println!("Registering {}", name);
        self.registry.insert(name, Box::new(constructor));
    }

    pub fn create_node(
        &self,
        node_name: String,
        config: &Value,
    ) -> Option<Box<dyn Process + 'static>> {
        // if a template is imported with the key "template" in the config, merge them together, with the config taking precedence
        let mut config = config.as_object().unwrap().clone();
        if let Some(template_name) = config.get("template") {
            let template_name = template_name.as_str().unwrap();
            let template = self.templates.get(template_name).expect("Unknown template");
            let template = template.as_object().unwrap();
            for (key, value) in template {
                if !config.contains_key(key) {
                    config.insert(key.clone(), value.clone());
                }
            }
        }
        // if there is no driver field we just skip this node
        if !config.contains_key("driver") {
            return None;
        }
        let driver = config["driver"]
            .as_str()
            .expect("driver field must be a string");
        dbg!(driver);
        dbg!(self.registry.keys());
        let constructor = self
            .registry
            .get(driver)
            .expect(format!("Unknown driver {}", driver).as_str());
        Some(constructor(node_name, config))
    }
}
