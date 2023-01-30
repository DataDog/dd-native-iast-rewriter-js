/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
extern crate base64;

use crate::{
    rewriter::{print_js, rewrite_js, Config},
    telemetry::TelemetryVerbosity,
    util::rnd_string,
    visitor::{self, csi_methods::CsiMethods},
};

use napi::{Error, Status};

#[napi(object)]
#[derive(Debug)]
pub struct CsiMethod {
    pub src: String,
    pub dst: Option<String>,
    pub operator: Option<bool>,
}

#[napi(object)]
#[derive(Debug)]
pub struct RewriterConfig {
    pub chain_source_map: Option<bool>,
    pub comments: Option<bool>,
    pub local_var_prefix: Option<String>,
    pub csi_methods: Option<Vec<CsiMethod>>,
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
        }
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
        });
        Self {
            config: rewriter_config.to_config(),
        }
    }

    #[napi]
    pub fn rewrite(&self, code: String, file: String) -> napi::Result<String> {
        rewrite_js(code, &file, &self.config)
            .map(|result| print_js(&result, self.config.chain_source_map))
            .map_err(|e| Error::new(Status::Unknown, format!("{}", e)))
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
