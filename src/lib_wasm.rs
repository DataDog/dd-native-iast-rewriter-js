/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
extern crate base64;

use std::{
    collections::HashMap,
    io::{Cursor, Read},
    path::{Path, PathBuf},
};

use crate::{
    rewriter::{print_js, rewrite_js, Config},
    telemetry::{Telemetry, TelemetryVerbosity},
    transform::transform_status::TransformStatus,
    util::{rnd_string, FileReader},
    visitor::{self, csi_methods::CsiMethods},
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::wasm_bindgen, JsError, JsValue};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CsiMethod {
    pub src: String,
    pub dst: Option<String>,
    pub operator: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RewriterConfig {
    pub chain_source_map: Option<bool>,
    pub comments: Option<bool>,
    pub local_var_prefix: Option<String>,
    pub csi_methods: Option<Vec<CsiMethod>>,
    pub telemetry_verbosity: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    pub content: String,
    pub metrics: Option<Metrics>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
    pub status: String,
    pub instrumented_propagation: u32,
    pub file: String,
    pub propagation_debug: Option<HashMap<String, u32>>,
}

impl RewriterConfig {
    fn get_csi_methods(&self) -> CsiMethods {
        match &self.csi_methods {
            Some(methods) => CsiMethods::new(
                &methods
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

            None => CsiMethods::empty(),
        }
    }

    fn to_config(&self) -> Config {
        Config {
            chain_source_map: self.chain_source_map.unwrap_or(false),
            print_comments: self.comments.unwrap_or(false),
            local_var_prefix: self
                .local_var_prefix
                .clone()
                .unwrap_or_else(|| rnd_string(6)),
            csi_methods: self.get_csi_methods(),
            verbosity: TelemetryVerbosity::parse(self.telemetry_verbosity.clone()),
        }
    }
}

#[wasm_bindgen]
pub struct Rewriter {
    config: Config,
}

#[wasm_bindgen(module = "fs")]
extern "C" {
    #[wasm_bindgen(js_name = readFileSync, catch)]
    fn read_file(path: &str) -> anyhow::Result<JsValue, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(module = "path")]
extern "C" {
    #[wasm_bindgen(js_name = dirname, catch)]
    fn dirname(s: &str) -> anyhow::Result<JsValue, JsValue>;
}

struct WasmFileReader {}
impl FileReader<Cursor<Vec<u8>>> for WasmFileReader {
    fn read(&self, path: &Path) -> std::io::Result<Cursor<Vec<u8>>>
    where
        Cursor<Vec<u8>>: Read,
        Self: Sized,
    {
        read_file(path.to_str().unwrap())
            .map(|buffer| {
                let arr = js_sys::Uint8Array::new(&buffer);
                Cursor::new(arr.to_vec())
            })
            .map_err(|err| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Error reading source map from wasm {err:?}"),
                )
            })
    }

    fn parent(&self, path: &Path) -> Option<PathBuf> {
        match dirname(path.to_str().unwrap()) {
            Ok(parent) => Some(PathBuf::from(parent.as_string().unwrap().as_str())),
            Err(_) => None,
        }
    }

    fn log(&self, msg: String) {
        log(msg.as_str());
    }
}

#[wasm_bindgen]
impl Rewriter {
    #[wasm_bindgen(constructor)]
    pub fn new(config_js: JsValue) -> Self {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        let config = serde_wasm_bindgen::from_value::<RewriterConfig>(config_js);
        let rewriter_config: RewriterConfig = config.unwrap_or(RewriterConfig {
            chain_source_map: Some(false),
            comments: Some(false),
            local_var_prefix: None,
            csi_methods: None,
            telemetry_verbosity: Some("INFORMATION".to_string()),
        });
        Self {
            config: rewriter_config.to_config(),
        }
    }

    #[wasm_bindgen]
    pub fn rewrite(&mut self, code: String, file: String) -> anyhow::Result<JsValue, JsError> {
        let source_map_reader = WasmFileReader {};

        rewrite_js(code, &file, &self.config, &source_map_reader)
            .map(|result| Result {
                content: print_js(&result, &self.config),
                metrics: get_metrics(result.transform_status, file),
            })
            .as_ref()
            .map(|result| serde_wasm_bindgen::to_value(result).unwrap())
            .map_err(|e| JsError::new(&format!("{e}")))
    }

    #[wasm_bindgen(js_name = csiMethods)]
    pub fn csi_methods(&self) -> anyhow::Result<JsValue, JsError> {
        let csi_methods = &self.config.csi_methods;
        let dst_methods = csi_methods
            .methods
            .iter()
            .map(|csi_method| csi_method.dst.clone())
            .collect::<Vec<String>>();

        serde_wasm_bindgen::to_value(&dst_methods).map_err(|e| JsError::new(&format!("{e}")))
    }
}

fn get_metrics(status: Option<TransformStatus>, file: String) -> Option<Metrics> {
    if let Some(transform_status) = status {
        return Some(Metrics {
            status: transform_status.status.to_string(),
            instrumented_propagation: transform_status.telemetry.get_instrumented_propagation(),
            propagation_debug: transform_status.telemetry.get_propagation_debug(),
            file,
        });
    }
    None
}
