/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/

#[cfg(test)]
mod tests {
    use spectral::{assert_that, prelude::ContainingIntoIterAssertions};

    use crate::tests::{get_hardcoded_secret_config, rewrite_js_with_config};

    #[test]
    fn test_literal_outside_block() -> Result<(), String> {
        let original_code = "const a = 'literal_literal';".to_string();

        let rewritten = rewrite_js_with_config(original_code, &get_hardcoded_secret_config())
            .map_err(|e| e.to_string())?;

        assert_that(&rewritten.hardcoded_secret_result.is_some());
        assert_that(&rewritten.hardcoded_secret_result.unwrap().matches)
            .contains("literal_literal".to_string());
        Ok(())
    }

    #[test]
    fn test_literal_inside_block() -> Result<(), String> {
        let original_code = "{ const a = 'literal_literal'; }".to_string();

        let rewritten = rewrite_js_with_config(original_code, &get_hardcoded_secret_config())
            .map_err(|e| e.to_string())?;

        assert_that(&rewritten.hardcoded_secret_result.is_some());
        assert_that(&rewritten.hardcoded_secret_result.unwrap().matches)
            .contains("literal_literal".to_string());
        Ok(())
    }

    #[test]
    fn test_literal_skipped_due_length() -> Result<(), String> {
        let original_code = "{ const a = 'literal'; }".to_string();

        let rewritten = rewrite_js_with_config(original_code, &get_hardcoded_secret_config())
            .map_err(|e| e.to_string())?;

        assert_that(&rewritten.hardcoded_secret_result.is_some());
        assert_that(&rewritten.hardcoded_secret_result.unwrap().matches).is_equal_to(vec![]);
        Ok(())
    }
}
