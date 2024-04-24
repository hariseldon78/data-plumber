use serde_json::Value;

pub struct Config {
    pub state_file: String,
    pub template_file: String,
}

impl Config {
    pub fn from(value: &Value) -> Config {
        Config {
            state_file: value["config"]["state_file"].as_str().unwrap_or("state.json").to_string(),
            template_file: value["config"]["template_file"].as_str().unwrap_or("template.json").to_string(),
        }
    }
}
