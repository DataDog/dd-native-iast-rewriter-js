#![deny(clippy::all)]
/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
mod rewriter;
mod transform;
mod util;
mod visitor;

#[cfg(test)]
mod tests;

#[cfg(feature = "wasm")]
mod lib_wasm;

#[macro_use]
#[cfg(not(feature = "wasm"))]
extern crate napi_derive;

#[cfg(not(feature = "wasm"))]
mod lib_napi;
use std::collections::HashMap;

use crate::rewriter::{print_js, rewrite_js};
use napi::{Error, Status};
use visitor::csi_methods::{CsiMethod, CsiMethods};

#[napi(object)]
#[derive(Debug)]
pub struct CsiMethodNapi {
    pub src: String,
    pub dst: Option<String>,
}

#[napi(object)]
#[derive(Debug)]
pub struct RewriterConfig {
    pub chain_source_map: Option<bool>,
    pub comments: Option<bool>,
    pub csi_methods: Option<Vec<CsiMethodNapi>>,
}

impl RewriterConfig {
    fn get_csi_methods(&self) -> CsiMethods {
        match &self.csi_methods {
            Some(methods_napi) => CsiMethods::new(
                &mut methods_napi
                    .iter()
                    .map(|m| CsiMethod::new(m.src.clone(), m.dst.clone()))
                    .collect::<Vec<CsiMethod>>(),
            ),
            None => CsiMethods::empty(),
        }
    }
}

#[napi]
pub struct Rewriter {
    config: RewriterConfig,
}

#[napi]
impl Rewriter {
    #[napi(constructor)]
    pub fn new(config: Option<RewriterConfig>) -> Self {
        let rewriter_config: RewriterConfig = config.unwrap_or(RewriterConfig {
            chain_source_map: Some(false),
            comments: Some(false),
            csi_methods: None,
        });
        Self {
            config: rewriter_config,
        }
    }

    #[napi]
    pub fn rewrite(&self, code: String, file: String) -> napi::Result<String> {
        rewrite_js(
            code,
            file,
            self.config.comments.unwrap_or(false),
            &self.config.get_csi_methods(),
        )
        .map(|result| print_js(result, self.config.chain_source_map.unwrap_or(false)))
        .map_err(|e| Error::new(Status::Unknown, format!("{}", e)))
    }

    #[napi]
    pub fn csi_methods(&self) -> napi::Result<Vec<String>> {
        let csi_methods = &self.config.get_csi_methods();

        Ok(csi_methods
            .methods
            .iter()
            .map(|csi_method| csi_method.get_dst())
            .collect())
    }
}
