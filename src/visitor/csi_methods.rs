/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use std::collections::{hash_map::Keys, HashMap};

#[derive(Clone)]
pub struct CsiMethods {
    pub methods: HashMap<&'static str, Vec<&'static str>>,
}

impl CsiMethods {
    pub fn new() -> Self {
        CsiMethods {
            methods: HashMap::from([("String.prototype", vec!["substring"])]),
        }
    }
    pub fn contains(&self, method_name: &String) -> bool {
        let name = &method_name.as_str();
        self.methods.values().any(|m| m.contains(name))
    }

    pub fn class_name_prototype_keys(&self) -> Keys<&str, Vec<&str>> {
        self.methods.keys()
    }
}
