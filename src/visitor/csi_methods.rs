use std::collections::HashMap;

use super::visitor_util::DD_PLUS_OPERATOR;

/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/

#[derive(Clone)]
pub struct CsiMethod {
    class_name: String,
    method_name: String,
}

impl CsiMethod {
    pub fn simple(method_name: &str) -> Self {
        CsiMethod {
            class_name: "".to_string(),
            method_name: method_name.to_string(),
        }
    }

    pub fn rewritten_name(&self) -> String {
        if self.class_name.is_empty() {
            self.method_name.clone()
        } else {
            format!(
                "{}_{}",
                self.class_name.to_lowercase().replace(".prototype", ""),
                self.method_name
            )
        }
    }
}

#[derive(Clone)]
pub struct CsiMethods {
    pub class_names: Vec<String>,
    pub methods: Vec<CsiMethod>,
}

impl CsiMethods {
    pub fn from(csi_methods: &Option<HashMap<String, Vec<String>>>) -> Self {
        match csi_methods {
            Some(methods) => CsiMethods::new(methods),
            None => Self::empty(),
        }
    }

    pub fn empty() -> Self {
        CsiMethods {
            class_names: vec![],
            methods: vec![],
        }
    }

    pub fn new(csi_methods: &HashMap<String, Vec<String>>) -> Self {
        let mut methods = vec![CsiMethod::simple(DD_PLUS_OPERATOR)];
        let mut class_names = Vec::new();
        register(&mut methods, &mut class_names, csi_methods);

        CsiMethods {
            class_names,
            methods,
        }
    }

    pub fn get(&self, method_name: &str) -> Option<&CsiMethod> {
        self.methods
            .iter()
            .find(|csi_method| csi_method.method_name == method_name)
    }
}

fn register(
    methods: &mut Vec<CsiMethod>,
    class_names: &mut Vec<String>,
    csi_methods: &HashMap<String, Vec<String>>,
) {
    for def in csi_methods {
        let class_name = def.0.to_string();
        let method_names = def.1;
        for method_name_str in method_names {
            let csi_method = CsiMethod {
                class_name: class_name.clone(),
                method_name: method_name_str.to_string(),
            };
            methods.push(csi_method);
        }
        class_names.push(class_name);
    }
}
