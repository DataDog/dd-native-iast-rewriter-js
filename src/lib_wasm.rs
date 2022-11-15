/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
extern crate base64;

use crate::rewriter::{print_js, rewrite_js};

use wasm_bindgen::{prelude::wasm_bindgen, JsError, JsValue};

#[wasm_bindgen]
pub struct RewriterConfig {
    #[wasm_bindgen(js_name = chainSourceMap)]
    pub chain_source_map: Option<bool>,
    pub comments: Option<bool>,

    #[wasm_bindgen(skip)]
    pub local_var_prefix: Option<String>,
}

#[wasm_bindgen]
impl RewriterConfig {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        RewriterConfig {
            chain_source_map: None,
            comments: None,
            local_var_prefix: None,
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
        )
        .map(|result| print_js(result, self.config.chain_source_map.unwrap_or(false)))
        .map_err(|e| JsError::new(&format!("{}", e)))
    }
}
