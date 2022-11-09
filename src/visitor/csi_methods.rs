/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use super::visitor_util::DD_PLUS_OPERATOR;

#[derive(Clone)]
pub struct CsiMethod {
    src: String,
    dst: Option<String>,
}

impl CsiMethod {
    pub fn new(src: String, dst: Option<String>) -> Self {
        CsiMethod { src, dst }
    }

    pub fn get_dst(&self) -> String {
        match &self.dst {
            Some(dst) => dst.clone(),
            None => format!("_{}", self.src),
        }
    }
}

#[derive(Clone)]
pub struct CsiMethods {
    pub methods: Vec<CsiMethod>,
}

impl CsiMethods {
    pub fn new(csi_methods: &mut Vec<CsiMethod>) -> Self {
        let mut methods = vec![];
        if !csi_methods.is_empty() {
            methods.push(CsiMethod::new(
                DD_PLUS_OPERATOR.to_string(),
                Some(DD_PLUS_OPERATOR.to_string()),
            ));
            methods.append(csi_methods);
        }
        CsiMethods { methods }
    }

    pub fn empty() -> Self {
        CsiMethods { methods: vec![] }
    }

    pub fn get(&self, method_name: &str) -> Option<&CsiMethod> {
        self.methods
            .iter()
            .find(|csi_method| csi_method.src == method_name)
    }

    pub fn plus_operator_is_enabled(&self) -> bool {
        self.get(DD_PLUS_OPERATOR).is_some()
    }
}
