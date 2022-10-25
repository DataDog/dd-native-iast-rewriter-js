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
            "String.prototype",
            vec!["substring"],
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

fn register(
    methods: &mut Vec<CsiMethod>,
    class_names: &mut Vec<String>,
    class_name: &str,
    method_names: Vec<&'static str>,
) {
    class_names.push(class_name.to_string());

    for method_name in method_names {
        methods.push(CsiMethod {
            class_name: class_name.to_string(),
            method_name: method_name.to_string(),
        })
    }
}
