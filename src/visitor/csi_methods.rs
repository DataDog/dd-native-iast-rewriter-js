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
    pub fn full_name(&self) -> String {
        format!(
            "{}_{}",
            self.class_name.to_lowercase().replace(".prototype", ""),
            self.method_name
        )
    }
}

#[derive(Clone)]
pub struct CsiMethods {
    pub class_names: Vec<String>,
    pub methods: Vec<CsiMethod>,
}

impl CsiMethods {
    pub fn new() -> Self {
        let mut methods = Vec::new();
        let mut class_names = Vec::new();
        register(
            &mut methods,
            &mut class_names,
            &[(
                "String.prototype",
                &[
                    "substring",
                    "trim",
                    "trimStart",
                    "trimEnd",
                    "toLowerCase",
                    "toLocaleLowerCase",
                    "toUpperCase",
                    "toLocaleUpperCase",
                    "replace",
                    "replaceAll",
                    "slice",
                    "concat",
                ],
            )],
        );

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

// a little uggly? :/
fn register(
    methods: &mut Vec<CsiMethod>,
    class_names: &mut Vec<String>,
    method_defs: &[(&str, &[&str])], // [(class_name, [method_names])]
) {
    for def in method_defs {
        let class_name = def.0;
        let method_names = def.1;
        class_names.push(class_name.to_string());
        for method_name in method_names {
            methods.push(CsiMethod {
                class_name: class_name.to_string(),
                method_name: method_name.to_string(),
            })
        }
    }
}
