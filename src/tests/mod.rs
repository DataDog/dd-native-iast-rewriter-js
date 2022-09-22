/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use crate::visitor::visitor_util::DD_LOCAL_VAR_NAME_HASH_ENV_NAME;
use std::{env, path::PathBuf};

mod binary_assignation_test;
mod binary_expression_test;
mod source_map_test;
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