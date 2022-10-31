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

#[macro_use]
extern crate napi_derive;
extern crate base64;

use crate::{
    rewriter::{print_js, rewrite_js},
    visitor::visitor_util::DD_PLUS_OPERATOR,
};
use napi::{Error, Status};
use visitor::csi_methods::{CsiExclusions, CsiMethods};

#[napi(object)]
#[derive(Debug)]
pub struct RewriterConfig {
    pub chain_source_map: Option<bool>,
    pub comments: Option<bool>,
    pub csi_exclusions: Option<Vec<String>>,
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
            csi_exclusions: None,
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
            CsiExclusions::from(&self.config.csi_exclusions),
        )
        .map(|result| print_js(result, self.config.chain_source_map.unwrap_or(false)))
        .map_err(|e| Error::new(Status::Unknown, format!("{}", e)))
    }

    #[napi]
    pub fn csi_methods(&self) -> napi::Result<Vec<String>> {
        let csi_exclusions = CsiExclusions::from(&self.config.csi_exclusions);
        let csi_methods = CsiMethods::new(&csi_exclusions);

        let mut method_names = vec![DD_PLUS_OPERATOR.to_string()];
        let mut csi_method_names = csi_methods
            .methods
            .iter()
            .map(|csi_method| csi_method.rewritten_name())
            .collect();
        method_names.append(&mut csi_method_names);
        Ok(method_names)
    }
}
