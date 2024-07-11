/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use crate::{
    rewriter::{Config, RewrittenOutput},
    telemetry::TelemetryVerbosity,
    util::DefaultFileReader,
    visitor::csi_methods::{CsiMethod, CsiMethods},
};
use anyhow::Error;
use std::path::PathBuf;

mod binary_assignation_test;
mod binary_expression_test;
mod literal_test;
mod source_map_test;
mod string_method_test;
mod telemetry_test;
mod template_literal_test;

fn get_test_resources_folder() -> Result<PathBuf, String> {
    std::env::current_dir()
        .map(|cwd| cwd.join("test").join("resources"))
        .map_err(|e| e.to_string())
}

fn rewrite_js(code: String, file: String) -> Result<RewrittenOutput, Error> {
    let source_map_reader = DefaultFileReader {};
    crate::rewriter::rewrite_js(code, &file, &get_default_config(false), &source_map_reader)
}

fn rewrite_js_with_telemetry_verbosity(
    code: String,
    file: String,
    verbosity: TelemetryVerbosity,
) -> Result<RewrittenOutput, Error> {
    let source_map_reader = DefaultFileReader {};
    crate::rewriter::rewrite_js(
        code,
        &file,
        &get_default_config_with_verbosity(false, verbosity),
        &source_map_reader,
    )
}

fn rewrite_js_with_csi_methods(
    code: String,
    file: String,
    csi_methods: &CsiMethods,
) -> Result<RewrittenOutput, Error> {
    let source_map_reader = DefaultFileReader {};
    crate::rewriter::rewrite_js(
        code,
        &file,
        &Config {
            chain_source_map: false,
            print_comments: false,
            local_var_prefix: "test".to_string(),
            csi_methods: csi_methods.clone(),
            verbosity: TelemetryVerbosity::Information,
            literals: false,
        },
        &source_map_reader,
    )
}

fn rewrite_js_with_config(code: String, config: &Config) -> Result<RewrittenOutput, Error> {
    let source_map_reader = DefaultFileReader {};
    crate::rewriter::rewrite_js(code, "test.js", config, &source_map_reader)
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
        csi_from_str("replace", None),
    ];
    CsiMethods::new(&mut methods)
}

fn get_default_config(print_comments: bool) -> Config {
    get_default_config_with_verbosity(print_comments, TelemetryVerbosity::Debug)
}

fn get_default_config_with_verbosity(
    print_comments: bool,
    verbosity: TelemetryVerbosity,
) -> Config {
    Config {
        chain_source_map: false,
        print_comments,
        local_var_prefix: "test".to_string(),
        csi_methods: get_default_csi_methods(),
        verbosity,
        literals: false,
    }
}

fn get_chained_and_print_comments_config() -> Config {
    Config {
        chain_source_map: true,
        print_comments: true,
        local_var_prefix: "test".to_string(),
        csi_methods: get_default_csi_methods(),
        verbosity: TelemetryVerbosity::Debug,
        literals: false,
    }
}

fn get_literals_config() -> Config {
    Config {
        chain_source_map: true,
        print_comments: true,
        local_var_prefix: "test".to_string(),
        csi_methods: get_default_csi_methods(),
        verbosity: TelemetryVerbosity::Debug,
        literals: true,
    }
}

fn csi_from_str(src: &str, dst: Option<&str>) -> CsiMethod {
    let dst_string = match dst {
        Some(str) => Some(String::from(str)),
        None => None,
    };
    CsiMethod::new(String::from(src), dst_string, false, false)
}

fn csi_op_from_str(src: &str, dst: Option<&str>) -> CsiMethod {
    let dst_string = match dst {
        Some(str) => Some(String::from(str)),
        None => None,
    };
    CsiMethod::new(String::from(src), dst_string, true, false)
}
