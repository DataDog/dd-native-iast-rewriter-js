/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
extern crate base64;

use std::collections::HashMap;

use crate::{
    rewriter::{print_js, rewrite_js, Config},
    telemetry::{Telemetry, TelemetryVerbosity},
    transform::transform_status::TransformStatus,
    util::{rnd_string, DefaultFileReader},
    visitor::{self, csi_methods::CsiMethods, literal_visitor},
};

use napi::{Error, Status};

#[napi(object)]
#[derive(Debug)]
pub struct CsiMethod {
    pub src: String,
    pub dst: Option<String>,
    pub operator: Option<bool>,
    pub allowed_without_callee: Option<bool>,
}

#[napi(object)]
#[derive(Debug)]
pub struct RewriterConfig {
    pub chain_source_map: Option<bool>,
    pub comments: Option<bool>,
    pub local_var_prefix: Option<String>,
    pub csi_methods: Option<Vec<CsiMethod>>,
    pub literals: Option<bool>,
}

impl RewriterConfig {
    fn get_csi_methods(&self) -> CsiMethods {
        match &self.csi_methods {
            Some(methods_napi) => CsiMethods::new(
                &methods_napi
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
            verbosity: TelemetryVerbosity::Information,
            literals: self.literals.unwrap_or(true),
        }
    }
}

#[napi(object, js_name = "Result")]
#[derive(Debug)]
pub struct RewriteResult {
    pub content: String,
    pub metrics: Option<Metrics>,
    pub literals_result: Option<LiteralsResult>,
}

#[napi(object)]
#[derive(Debug)]
pub struct Metrics {
    pub status: String,
    pub instrumented_propagation: u32,
    pub file: String,
    pub propagation_debug: Option<HashMap<String, u32>>,
}

#[napi(object)]
#[derive(Debug)]
pub struct LiteralsResult {
    pub file: String,
    pub literals: Vec<LiteralInfo>,
}

#[napi(object)]
#[derive(Debug)]
pub struct LiteralLocation {
    pub ident: Option<String>,
    pub line: i32,
    pub column: i32,
}

#[napi(object)]
#[derive(Debug)]
pub struct LiteralInfo {
    pub value: String,
    pub locations: Vec<LiteralLocation>,
}

impl LiteralInfo {
    fn from(literals: Vec<literal_visitor::LiteralInfo>) -> Vec<LiteralInfo> {
        literals
            .iter()
            .map(|literal| LiteralInfo {
                value: literal.value.clone(),
                locations: literal
                    .locations
                    .iter()
                    .map(|location| LiteralLocation {
                        ident: location.ident.clone(),
                        line: location.line as i32,
                        column: location.column as i32,
                    })
                    .collect(),
            })
            .collect()
    }
}

#[napi]
pub struct Rewriter {
    config: Config,
}

#[napi]
impl Rewriter {
    #[napi(constructor)]
    pub fn new(config: Option<RewriterConfig>) -> Self {
        let rewriter_config: RewriterConfig = config.unwrap_or(RewriterConfig {
            chain_source_map: Some(false),
            comments: Some(false),
            local_var_prefix: None,
            csi_methods: None,
            literals: Some(true),
        });
        Self {
            config: rewriter_config.to_config(),
        }
    }

    #[napi]
    pub fn rewrite(&self, code: String, file: String) -> napi::Result<RewriteResult> {
        let default_file_reader = DefaultFileReader {};

        rewrite_js(code, &file, &self.config, &default_file_reader)
            .map(|result| RewriteResult {
                content: print_js(&result, &self.config),
                metrics: get_metrics(result.transform_status, &file),
                literals_result: match result.literals_result {
                    Some(literals_result) => Some(LiteralsResult {
                        file,
                        literals: LiteralInfo::from(literals_result.literals),
                    }),
                    _ => None,
                },
            })
            .map_err(|e| Error::new(Status::Unknown, format!("{e}")))
    }

    #[napi]
    pub fn csi_methods(&self) -> napi::Result<Vec<String>> {
        let csi_methods = &self.config.csi_methods;

        Ok(csi_methods
            .methods
            .iter()
            .map(|csi_method| csi_method.dst.clone())
            .collect())
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
