use crate::state::{read_config_field, Process, State, Table};
use itertools::Itertools;
use serde_json::Value;
use std::{cmp::Ordering, fs::File, result::Result};

pub struct OutputCompare {
    node_name: String,
    input1: String,
    input2: String,
    path: String,
    identity_field: String,
}

impl Process for OutputCompare {
    fn from_config(node_name: String, config: &Value) -> Self {
        OutputCompare {
            node_name,
            input1: read_config_field(config, "input1"),
            input2: read_config_field(config, "input2"),
            path: read_config_field(config, "path"),
            identity_field: read_config_field(config, "identity_field"),
        }
    }
    fn run(&self, state: &mut State) {
        let t1 = state.find_table(self.input1.as_str()).unwrap();
        let t2 = state.find_table(self.input2.as_str()).unwrap();

        let mut differences: Vec<String> = vec![];
        let mut sorted1 = t1.records.iter().sorted_by(|a, b| {
            Ord::cmp(
                a.fields[self.identity_field.as_str()].to_string().as_str(),
                b.fields[self.identity_field.as_str()].to_string().as_str(),
            )
        });
        let mut sorted2 = t2.records.iter().sorted_by(|a, b| {
            Ord::cmp(
                a.fields[self.identity_field.as_str()].to_string().as_str(),
                b.fields[self.identity_field.as_str()].to_string().as_str(),
            )
        });
        let mut it1 = sorted1.next();
        let mut it2 = sorted2.next();
        loop {
            match (it1, it2) {
                (Some(r1), Some(r2)) => {
                    match Ord::cmp(
                        r1.fields[self.identity_field.as_str()].to_string().as_str(),
                        r2.fields[self.identity_field.as_str()].to_string().as_str(),
                    ) {
                        Ordering::Less => {
                            differences.push(format!(
                                "Extra in {}: {} - a",
                                self.input1,
                                r1.fields[self.identity_field.as_str()]
                            ));
                            it1 = sorted1.next();
                            ()
                        }
                        Ordering::Greater => {
                            differences.push(format!(
                                "Extra in {}: {} - b",
                                self.input2,
                                r2.fields[self.identity_field.as_str()]
                            ));
                            it2 = sorted2.next();
                            ()
                        }
                        Ordering::Equal => {
                            //same key
                            // find fields that are different
                            for (k, v) in r1.fields.iter() {
                                if r2.fields.contains_key(k) {
                                    if r2.fields[k] != *v {
                                        differences.push(format!(
                                            "Mismatch: id: {}, field: {}, ({}) {} != ({}) {}",
                                            r1.fields[self.identity_field.as_str()],
                                            k,
                                            self.input1,
                                            v,
                                            self.input2,
                                            r2.fields[k]
                                        ));
                                    }
                                } else {
                                    differences.push(format!(
                                        "Missing field in {}: id: {}, field: {}",
                                        self.input2,
                                        r1.fields[self.identity_field.as_str()],
                                        k
                                    ));
                                }
                            }
                            // find fields that are in r2 but not in r1
                            for (k, v) in r2.fields.iter() {
                                if !r1.fields.contains_key(k) {
                                    differences.push(format!(
                                        "Missing field in {}: id: {}, field: {}",
                                        self.input1,
                                        r2.fields[self.identity_field.as_str()],
                                        k
                                    ));
                                }
                            }
                            it1 = sorted1.next();
                            it2 = sorted2.next();
                            ()
                        }
                    }
                }
                (Some(r1), None) => {
                    differences.push(format!(
                        "Extra in {}: {} - c",
                        self.input1,
                        r1.fields[self.identity_field.as_str()]
                    ));
                    it1 = sorted1.next();
                    ()
                }
                (None, Some(r2)) => {
                    differences.push(format!(
                        "Extra in {}: {} - d",
                        self.input2,
                        r2.fields[self.identity_field.as_str()]
                    ));
                    it2 = sorted2.next();
                    ()
                }
                (None, None) => {
                    break;
                }
            }
        }
        let output = differences.join("\n");
        std::fs::write(&self.path, output).unwrap();
    }
}
