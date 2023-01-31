/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/

#[cfg(test)]
mod tests {
    use spectral::{assert_that, option::OptionAssertions, prelude::HashMapAssertions};

    use crate::{
        telemetry::{IastTelemetry, Telemetry, TelemetryVerbosity},
        tests::{rewrite_js, rewrite_js_with_telemetry_verbosity},
        transform::transform_status::Status,
    };

    const SUBSTRING: &str = "substring";

    #[test]
    fn test_debug_vebosity_telemetry() -> Result<(), String> {
        let original_code = "{const a = b.substring(1);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;

        assert_that(&rewritten.transform_status).is_some();

        let status = rewritten.transform_status.unwrap();
        assert_that(&status.status).is_equal_to(Status::Modified);
        assert_that(&status.telemetry.get_instrumented_propagation()).is_equal_to(&1);
        assert_that(&status.telemetry.get_propagation_debug()).is_some();
        let propagation_debug = status.telemetry.get_propagation_debug().unwrap();
        assert_that(&propagation_debug).contains_key(SUBSTRING.to_string());
        assert_that(&propagation_debug.get(SUBSTRING).unwrap()).is_equal_to(&1);

        Ok(())
    }

    #[test]
    fn test_off_vebosity_telemetry() -> Result<(), String> {
        let original_code = "{const a = b.substring(1);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten =
            rewrite_js_with_telemetry_verbosity(original_code, js_file, TelemetryVerbosity::Off)
                .map_err(|e| e.to_string())?;

        assert_that(&rewritten.transform_status).is_some();

        let status = rewritten.transform_status.unwrap();
        assert_that(&status.status).is_equal_to(Status::Modified);
        assert_that(&status.telemetry.get_instrumented_propagation()).is_equal_to(&0);
        assert_that(&status.telemetry.get_propagation_debug()).is_none();

        Ok(())
    }

    #[test]
    fn test_information_vebosity_telemetry() -> Result<(), String> {
        let original_code = "{const a = b.substring(1);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js_with_telemetry_verbosity(
            original_code,
            js_file,
            TelemetryVerbosity::Information,
        )
        .map_err(|e| e.to_string())?;

        assert_that(&rewritten.transform_status).is_some();

        let status = rewritten.transform_status.unwrap();
        assert_that(&status.status).is_equal_to(Status::Modified);
        assert_that(&status.telemetry.get_instrumented_propagation()).is_equal_to(&1);
        assert_that(&status.telemetry.get_propagation_debug()).is_none();

        Ok(())
    }

    #[test]
    fn test_debug_vebosity_telemetry_with_multiple_calls() -> Result<(), String> {
        let original_code =
            "{const a = b.substring(1); const c = a.trim().substring(2);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;

        assert_that(&rewritten.transform_status).is_some();

        let status = rewritten.transform_status.unwrap();
        assert_that(&status.status).is_equal_to(Status::Modified);
        assert_that(&status.telemetry.get_instrumented_propagation()).is_equal_to(&3);
        assert_that(&status.telemetry.get_propagation_debug()).is_some();
        let propagation_debug = status.telemetry.get_propagation_debug().unwrap();
        assert_that(&propagation_debug).contains_key(SUBSTRING.to_string());
        assert_that(&propagation_debug.get(SUBSTRING).unwrap()).is_equal_to(&2);

        assert_that(&propagation_debug).contains_key("trim".to_string());
        assert_that(&propagation_debug.get("trim").unwrap()).is_equal_to(&1);

        Ok(())
    }

    #[test]
    fn test_debug_vebosity_telemetry_with_ignored() -> Result<(), String> {
        let original_code = "{const a = b.not_configured_method(1);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;

        assert_that(&rewritten.transform_status).is_some();

        let status = rewritten.transform_status.unwrap();
        assert_that(&status.status).is_equal_to(Status::NotModified);
        assert_that(&status.telemetry.get_instrumented_propagation()).is_equal_to(&0);
        assert_that(&status.telemetry.get_propagation_debug()).is_some();
        let propagation_debug = status.telemetry.get_propagation_debug().unwrap();
        assert_that(&propagation_debug).does_not_contain_key("not_configured_method".to_string());

        Ok(())
    }
}
