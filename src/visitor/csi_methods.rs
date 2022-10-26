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
    pub fn rewritten_name(&self) -> String {
        format!(
            "{}_{}",
            self.class_name.to_lowercase().replace(".prototype", ""),
            self.method_name
        )
    }

    pub fn full_name(&self) -> String {
        format!("{}.{}", self.class_name, self.method_name)
    }
}

#[derive(Clone)]
pub struct CsiMethods {
    pub class_names: Vec<String>,
    pub methods: Vec<CsiMethod>,
}

impl CsiMethods {
    pub fn new(csi_exclusions: &CsiExclusions) -> Self {
        let mut methods = Vec::new();
        let mut class_names = Vec::new();
        register(
            &mut methods,
            &mut class_names,
            csi_exclusions,
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
    csi_exclusions: &CsiExclusions,
    method_defs: &[(&str, &[&str])], // [(class_name, [method_names])]
) {
    for def in method_defs {
        let class_name = def.0.to_string();
        let method_names = def.1;
        let mut add_class_name = false;
        for method_name_str in method_names {
            let csi_method = CsiMethod {
                class_name: class_name.clone(),
                method_name: method_name_str.to_string(),
            };
            if csi_exclusions.is_excluded(&csi_method.full_name()) {
                continue;
            }
            methods.push(csi_method);
            add_class_name = true
        }

        if add_class_name {
            class_names.push(class_name);
        }
    }
}

pub struct CsiExclusions {
    exclusions: Vec<String>,
}

impl CsiExclusions {
    pub fn from(csi_exclusions: &Option<Vec<String>>) -> Self {
        match csi_exclusions {
            Some(exclusions) => CsiExclusions {
                exclusions: exclusions.clone(),
            },
            None => CsiExclusions::empty(),
        }
    }

    pub fn empty() -> Self {
        CsiExclusions {
            exclusions: Vec::new(),
        }
    }

    pub fn is_excluded(&self, method_name: &String) -> bool {
        self.exclusions.contains(method_name)
    }
}
