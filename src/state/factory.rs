use crate::state::State;
use serde_json::Value;

pub trait Process {
    fn from_config(node_name: String,config: &Value) -> Self;
    fn run(&self,state:&mut State);
}

// struct Factory; 
// impl Factory {
//     pub fn create(driver: &str) -> Box<dyn Node> {
//         match driver {
//             "input::mysql" => (),
//             "output::sql-inserts" => (),
//             _ => panic!("Unknown driver: {}", driver),
//         }
//     }
// }
