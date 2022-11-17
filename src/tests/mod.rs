/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use crate::rewriter::RewrittenOutput;
use anyhow::Error;
use std::path::PathBuf;
use crate::{
    rewriter::RewrittenOutput,
    visitor::{
        csi_methods::{CsiMethod, CsiMethods},
        visitor_util::DD_LOCAL_VAR_NAME_HASH_ENV_NAME,
    },
};
use anyhow::Error;
use std::{env, path::PathBuf};

mod binary_assignation_test;
mod binary_expression_test;
mod source_map_test;
mod string_method_test;
mod template_literal_test;

fn set_local_var() {
    match env::var(DD_LOCAL_VAR_NAME_HASH_ENV_NAME) {
        Err(_) => {
            env::set_var(DD_LOCAL_VAR_NAME_HASH_ENV_NAME, "test");
        }
        Ok(_) => {}
    }
}

fn get_test_resources_folder() -> Result<PathBuf, String> {
    std::env::current_dir()
        .map(|cwd| cwd.join("test").join("resources"))
        .map_err(|e| e.to_string())
}

fn rewrite_js(code: String, file: String) -> Result<RewrittenOutput, Error> {
    crate::rewriter::rewrite_js(code, file, false, Some("test".to_string()))
    crate::rewriter::rewrite_js(code, file, false, &get_default_csi_methods())
}

fn rewrite_js_with_csi_methods(
    code: String,
    file: String,
    csi_methods: &CsiMethods,
) -> Result<RewrittenOutput, Error> {
    crate::rewriter::rewrite_js(code, file, false, &csi_methods)
}

fn get_default_csi_methods() -> CsiMethods {
    let mut methods = vec![
        csi_op_from_str("plusOperator", None),
        csi_from_str("substring", Some("stringSubstring")),
        csi_from_str("trim", Some("stringTrim")),
        csi_from_str("trimStart", Some("stringTrim")),
        csi_from_str("trimEnd", Some("stringTrim")),
        csi_from_str("concat", Some("stringConcat")),
        csi_from_str("slice", None),
    ];
    CsiMethods::new(&mut methods)
}

fn csi_from_str(src: &str, dst: Option<&str>) -> CsiMethod {
    let dst_string = match dst {
        Some(str) => Some(String::from(str)),
        None => None,
    };
    CsiMethod::new(String::from(src), dst_string, false)
}

fn csi_op_from_str(src: &str, dst: Option<&str>) -> CsiMethod {
    let dst_string = match dst {
        Some(str) => Some(String::from(str)),
        None => None,
    };
    CsiMethod::new(String::from(src), dst_string, true)
}
