/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use super::visitor_util::DD_PLUS_OPERATOR;

#[derive(Clone)]
pub struct CsiMethod {
    pub src: String,
    pub dst: String,
    pub operator: bool,
}

impl CsiMethod {
    pub fn new(src: String, dst: Option<String>, operator: bool) -> Self {
        let dst = dst.unwrap_or_else(|| src.clone());
        CsiMethod { src, dst, operator }
    }
}

#[derive(Clone)]
pub struct CsiMethods {
    pub methods: Vec<CsiMethod>,
    pub plus_operator: Option<CsiMethod>,
}

impl CsiMethods {
    pub fn new(csi_methods: &[CsiMethod]) -> Self {
        let plus_operator = csi_methods
            .iter()
            .find(|csi_method| csi_method.operator && csi_method.src == DD_PLUS_OPERATOR);

        CsiMethods {
            methods: csi_methods.to_vec(),
            plus_operator: plus_operator.cloned(),
        }
    }

    pub fn empty() -> Self {
        CsiMethods {
            methods: vec![],
            plus_operator: None,
        }
    }

    pub fn get(&self, method_name: &str) -> Option<&CsiMethod> {
        self.methods
            .iter()
            .find(|csi_method| !csi_method.operator && csi_method.src == method_name)
    }

    pub fn plus_operator_is_enabled(&self) -> bool {
        self.plus_operator.is_some()
    }

    pub fn get_dd_plus_operator_name(&self) -> String {
        match &self.plus_operator {
            Some(csi_method) => csi_method.dst.clone(),
            _ => DD_PLUS_OPERATOR.to_string(),
        }
    }
}
