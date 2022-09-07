use std::env;

use crate::visitor::visitor_util::DD_LOCAL_VAR_NAME_HASH_ENV_NAME;

/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
mod binary_expression_test;
mod template_literal_test;

fn set_local_var() {
    match env::var(DD_LOCAL_VAR_NAME_HASH_ENV_NAME) {
        Err(_) => {
            env::set_var(DD_LOCAL_VAR_NAME_HASH_ENV_NAME, "test");
        }
        Ok(_) => {}
    }
}
