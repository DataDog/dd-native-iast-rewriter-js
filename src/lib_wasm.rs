/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
extern crate base64;

use crate::{
    rewriter::{parse_js, print_js, rewrite_js, Config},
    telemetry::{Telemetry, TelemetryVerbosity},
    tracer_logger::{self},
    transform::transform_status::TransformStatus,
    util::{rnd_string, FileReader},
    visitor::{self, csi_methods::CsiMethods, literal_visitor::LiteralsResult},
};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{Cursor, Read},
    path::{Path, PathBuf},
    sync::Arc,
};
use swc::{try_with_handler, Compiler, HandlerOpts};
use swc_common::{errors::ColorConfig, FileName, FilePathMapping};
use swc_ecma_ast::{Program, Stmt};
use wasm_bindgen::{prelude::wasm_bindgen, JsError, JsValue};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CsiMethod {
    pub src: String,
    pub dst: Option<String>,
    pub operator: Option<bool>,
    pub allowed_without_callee: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RewriterConfig {
    pub chain_source_map: Option<bool>,
    pub comments: Option<bool>,
    pub local_var_prefix: Option<String>,
    pub csi_methods: Option<Vec<CsiMethod>>,
    pub telemetry_verbosity: Option<String>,
    pub literals: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    pub content: String,
    pub metrics: Option<Metrics>,
    pub literals_result: Option<LiteralsResult>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
    pub status: String,
    pub instrumented_propagation: u32,
    pub file: String,
    pub propagation_debug: Option<HashMap<String, u32>>,
}

impl RewriterConfig {
    fn default() -> Self {
        RewriterConfig {
            chain_source_map: Some(false),
            comments: Some(false),
            local_var_prefix: None,
            csi_methods: None,
            telemetry_verbosity: Some("INFORMATION".to_string()),
            literals: Some(true),
        }
    }

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
                            m.allowed_without_callee.unwrap_or(false),
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
            literals: self.literals.unwrap_or(true),
            file_prefix_code: Vec::new(),
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
    {
        match path.to_str() {
            Some(path) => read_file(path)
                .map(|buffer| {
                    let arr = js_sys::Uint8Array::new(&buffer);
                    Cursor::new(arr.to_vec())
                })
                .map_err(|err| {
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Error reading source map from wasm {err:?}"),
                    )
                }),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Error reading source map. No path provided".to_string(),
            )),
        }
    }

    fn parent(&self, path: &Path) -> Option<PathBuf> {
        match path.to_str() {
            Some(path) => match dirname(path) {
                Ok(parent) => Some(PathBuf::from(
                    parent.as_string().unwrap_or_default().as_str(),
                )),
                Err(_) => None,
            },
            None => None,
        }
    }
}

fn generate_prefix_stmts(config: &Config) -> Vec<Stmt> {
    let template = ";if (typeof _ddiast === 'undefined') (function(globals){ const noop = (res) => res; globals._ddiast = globals._ddiast || { __CSI_METHODS__ }; }((1,eval)('this')));";

    let csi_methods_code = config
        .csi_methods
        .methods
        .iter()
        .map(|csi_method| format!("{}: noop", csi_method.dst))
        .collect::<Vec<_>>()
        .join(", ");

    let final_template = template.replace("__CSI_METHODS__", &csi_methods_code);

    let compiler = Compiler::new(Arc::new(swc_common::SourceMap::new(
        FilePathMapping::empty(),
    )));

    let handler_opts = HandlerOpts {
        color: ColorConfig::Never,
        skip_filename: false,
    };
    let program_result = try_with_handler(compiler.cm.clone(), handler_opts, |handler| {
        let source_file = compiler.cm.new_source_file(
            Arc::new(FileName::Real(PathBuf::from("inline.js".to_string()))),
            final_template.clone(),
        );

        parse_js(&source_file, handler, &compiler)
    });

    if let Ok(Program::Script(script)) = program_result {
        return script.body;
    }

    Vec::new()
}

#[wasm_bindgen]
impl Rewriter {
    #[wasm_bindgen(constructor)]
    pub fn new(config_js: JsValue) -> Self {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        let rewriter_config = serde_wasm_bindgen::from_value::<RewriterConfig>(config_js);
        let mut config: Config = rewriter_config
            .unwrap_or(RewriterConfig::default())
            .to_config();

        config
            .file_prefix_code
            .clone_from(&generate_prefix_stmts(&config));

        Self { config }
    }

    #[wasm_bindgen]
    pub fn rewrite(&mut self, code: String, file: String) -> anyhow::Result<JsValue, JsError> {
        let source_map_reader = WasmFileReader {};

        rewrite_js(code, &file, &self.config, &source_map_reader)
            .map(|result| Result {
                content: print_js(&result, &self.config),
                metrics: get_metrics(result.transform_status, &file),
                literals_result: result.literals_result,
            })
            .as_ref()
            .map(|result| {
                let status = &result.metrics;
                debug!("Rewritten {file}\n status {status:?}");

                serde_wasm_bindgen::to_value(result).unwrap()
            })
            .map_err(|e| {
                let error_msg = format!("{e}");
                error!("Error rewriting {}: {}", &file, &error_msg);
                JsError::new(&error_msg)
            })
    }

    #[wasm_bindgen(js_name = csiMethods)]
    pub fn csi_methods(&self) -> anyhow::Result<JsValue, JsError> {
        let csi_methods = &self.config.csi_methods;
        let dst_methods = csi_methods
            .methods
            .iter()
            .map(|csi_method| csi_method.dst.clone())
            .collect::<Vec<String>>();

        serde_wasm_bindgen::to_value(&dst_methods).map_err(|e| {
            let error_msg = format!("{e}");
            error!("Error getting csi methods: {}", &error_msg);
            JsError::new(&error_msg)
        })
    }

    #[wasm_bindgen(js_name = setLogger)]
    pub fn set_logger(&self, logger: &JsValue, level: &str) -> anyhow::Result<(), JsError> {
        tracer_logger::set_logger(logger, level)
            .map(|_| {
                log::log!(
                    log::max_level().to_level().unwrap_or(log::Level::Error),
                    "IAST rewriter logger configured OK"
                )
            })
            .map_err(|err| JsError::new(&format!("{err:?}")))
    }
}

fn get_metrics(status: Option<TransformStatus>, file: &str) -> Option<Metrics> {
    if let Some(transform_status) = status {
        return Some(Metrics {
            status: transform_status.status.to_string().to_lowercase(),
            instrumented_propagation: transform_status.telemetry.get_instrumented_propagation(),
            propagation_debug: transform_status.telemetry.get_propagation_debug(),
            file: file.to_owned(),
        });
    }
    None
}
