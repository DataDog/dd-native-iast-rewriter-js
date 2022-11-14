/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
extern crate base64;

use crate::rewriter::{print_js, rewrite_js};

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
pub struct RewriterConfig {
    pub chain_source_map: Option<bool>,
    pub comments: Option<bool>,
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
        });
        Self {
            config: rewriter_config,
        }
    }

    #[wasm_bindgen(catch, method)]
    pub fn rewrite(&self, code: String, file: String) -> anyhow::Result<String, JsValue> {
        rewrite_js(code, file, self.config.comments.unwrap_or(false))
            .map(|result| print_js(result, self.config.chain_source_map.unwrap_or(false)))
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
