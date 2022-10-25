/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use crate::rewriter::RewrittenOutput;
use anyhow::Error;
use std::path::PathBuf;

mod binary_assignation_test;
mod binary_expression_test;
mod source_map_test;
mod string_method_test;
mod template_literal_test;

fn get_test_resources_folder() -> Result<PathBuf, String> {
    std::env::current_dir()
        .map(|cwd| cwd.join("test").join("resources"))
        .map_err(|e| e.to_string())
}

fn rewrite_js(code: String, file: String) -> Result<RewrittenOutput, Error> {
    crate::rewriter::rewrite_js(code, file, false, Some("test".to_string()))
}
