/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
extern crate base64;

use crate::{
    rewriter::{print_js, rewrite_js},
    visitor::{self, csi_methods::CsiMethods},
};
use serde::Deserialize;
use wasm_bindgen::{prelude::wasm_bindgen, JsError, JsValue};

#[wasm_bindgen]
#[derive(Deserialize)]
pub struct CsiMethod {
    #[wasm_bindgen(skip)]
    pub src: String,

    #[wasm_bindgen(skip)]
    pub dst: Option<String>,

    pub operator: Option<bool>,
}

#[wasm_bindgen]
impl CsiMethod {
    #[wasm_bindgen(constructor)]
    pub fn new(src: String, dst: Option<String>, operator: Option<bool>) -> Self {
        CsiMethod { src, dst, operator }
    }

    #[wasm_bindgen(getter, js_name = src)]
    pub fn src(&self) -> String {
        self.src.clone()
    }

    #[wasm_bindgen(setter, js_name = src)]
    pub fn set_src(&mut self, src: String) {
        self.src = src
    }

    #[wasm_bindgen(getter, js_name = dst)]
    pub fn dst(&self) -> Option<String> {
        self.dst.clone()
    }

    #[wasm_bindgen(setter, js_name = dst)]
    pub fn set_dst(&mut self, dst: Option<String>) {
        self.dst = dst
    }
}

#[wasm_bindgen]
pub struct RewriterConfig {
    #[wasm_bindgen(js_name = chainSourceMap)]
    pub chain_source_map: Option<bool>,
    pub comments: Option<bool>,

    #[wasm_bindgen(skip)]
    pub local_var_prefix: Option<String>,

    #[wasm_bindgen(skip)]
    pub csi_methods: Option<JsValue>,
}

impl Default for RewriterConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl RewriterConfig {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        RewriterConfig {
            chain_source_map: None,
            comments: None,
            local_var_prefix: None,
            csi_methods: None,
        }
    }

    #[wasm_bindgen(getter, js_name = localVarPrefix)]
    pub fn local_var_prefix(&self) -> Option<String> {
        self.local_var_prefix.clone()
    }

    #[wasm_bindgen(setter, js_name = localVarPrefix)]
    pub fn set_local_var_prefix(&mut self, local_var_prefix: Option<String>) {
        self.local_var_prefix = local_var_prefix;
    }

    #[wasm_bindgen(setter, js_name = csiMethods)]
    pub fn set_csi_methods(&mut self, csi_methods: JsValue) {
        self.csi_methods = Some(csi_methods)
    }

    fn get_csi_methods(&self) -> CsiMethods {
        match &self.csi_methods {
            Some(methods_jsvalue) => {
                match serde_wasm_bindgen::from_value::<Vec<CsiMethod>>(methods_jsvalue.clone()) {
                    Ok(csi_methods) => CsiMethods::new(
                        &csi_methods
                            .iter()
                            .map(|m| {
                                visitor::csi_methods::CsiMethod::new(
                                    m.src.clone(),
                                    m.dst.clone(),
                                    m.operator.unwrap_or(false),
                                )
                            })
                            .collect::<Vec<visitor::csi_methods::CsiMethod>>(),
                    ),
                    _ => CsiMethods::empty(),
                }
            }

            None => CsiMethods::empty(),
        }
    }
}

#[wasm_bindgen]
pub struct Rewriter {
    config: RewriterConfig,
}

#[wasm_bindgen]
impl Rewriter {
    #[wasm_bindgen(constructor)]
    pub fn new(config: Option<RewriterConfig>) -> Self {
        let rewriter_config: RewriterConfig = config.unwrap_or(RewriterConfig {
            chain_source_map: Some(false),
            comments: Some(false),
            local_var_prefix: None,
            csi_methods: None,
        });
        Self {
            config: rewriter_config,
        }
    }

    #[wasm_bindgen]
    pub fn rewrite(&self, code: String, file: String) -> anyhow::Result<String, JsError> {
        rewrite_js(
            code,
            file,
            self.config.comments.unwrap_or(false),
            self.config.local_var_prefix.clone(),
            &self.config.get_csi_methods(),
        )
        .map(|result| print_js(result, self.config.chain_source_map.unwrap_or(false)))
        .map_err(|e| JsError::new(&format!("{}", e)))
    }

    #[wasm_bindgen(js_name = csiMethods)]
    pub fn csi_methods(&self) -> anyhow::Result<JsValue, JsError> {
        let csi_methods = &self.config.get_csi_methods();
        let dst_methods = csi_methods
            .methods
            .iter()
            .map(|csi_method| csi_method.dst.clone())
            .collect::<Vec<String>>();

        serde_wasm_bindgen::to_value(&dst_methods).map_err(|e| JsError::new(&format!("{}", e)))
    }
}
