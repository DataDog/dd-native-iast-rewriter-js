/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/

#[cfg(test)]
mod tests {
    use spectral::assert_that;

    use crate::tests::{get_hardcoded_secret_config, rewrite_js_with_config};

    #[test]
    fn test_literal_outside_block() -> Result<(), String> {
        let original_code = "const b = 1

            /*
            comment
             */

            const a = 'literal_literal';"
            .to_string();

        let rewritten = rewrite_js_with_config(original_code, &get_hardcoded_secret_config())
            .map_err(|e| e.to_string())?;

        assert_that(&rewritten.hardcoded_secret_result.is_some());
        let result = rewritten.hardcoded_secret_result.unwrap();
        let literal_info = result.literals.get(0).unwrap();

        assert_that(&literal_info.value).is_equal_to("literal_literal".to_string());
        assert_that(&literal_info.line).is_equal_to(Some(7));

        Ok(())
    }

    #[test]
    fn test_literal_inside_block() -> Result<(), String> {
        let original_code = "{ const secret = 'literal_literal'; }".to_string();

        let rewritten = rewrite_js_with_config(original_code, &get_hardcoded_secret_config())
            .map_err(|e| e.to_string())?;

        assert_that(&rewritten.hardcoded_secret_result.is_some());
        let result = rewritten.hardcoded_secret_result.unwrap();
        let literal_info = result.literals.get(0).unwrap();

        assert_that(&literal_info.value).is_equal_to("literal_literal".to_string());
        assert_that(&literal_info.line).is_equal_to(Some(1));

        Ok(())
    }

    #[test]
    fn test_literal_inside_obj_prop() -> Result<(), String> {
        let original_code = "{ const a = { secret: 'literal_literal' }; }".to_string();

        let rewritten = rewrite_js_with_config(original_code, &get_hardcoded_secret_config())
            .map_err(|e| e.to_string())?;

        assert_that(&rewritten.hardcoded_secret_result.is_some());
        let result = rewritten.hardcoded_secret_result.unwrap();
        let literal_info = result.literals.get(0).unwrap();

        assert_that(&literal_info.value).is_equal_to("literal_literal".to_string());
        assert_that(&literal_info.line).is_equal_to(Some(1));

        Ok(())
    }

    #[test]
    fn test_literal_as_argument() -> Result<(), String> {
        let original_code = "{ login('literal_literal') }".to_string();

        let rewritten = rewrite_js_with_config(original_code, &get_hardcoded_secret_config())
            .map_err(|e| e.to_string())?;

        assert_that(&rewritten.hardcoded_secret_result.is_some());
        let result = rewritten.hardcoded_secret_result.unwrap();
        let literal_info = result.literals.get(0).unwrap();

        assert_that(&literal_info.value).is_equal_to("literal_literal".to_string());
        assert_that(&literal_info.line).is_equal_to(Some(1));

        Ok(())
    }

    #[test]
    fn test_literal_skipped_due_length() -> Result<(), String> {
        let original_code = "{ const a = 'literal'; }".to_string();

        let rewritten = rewrite_js_with_config(original_code, &get_hardcoded_secret_config())
            .map_err(|e| e.to_string())?;

        assert_that(&rewritten.hardcoded_secret_result.is_some());
        assert_that(&rewritten.hardcoded_secret_result.unwrap().literals.len()).is_equal_to(0);
        Ok(())
    }

    #[test]
    fn test_require_literals_discarded() -> Result<(), String> {
        let original_code = "const a = require('literal_literal')".to_string();

        let rewritten = rewrite_js_with_config(original_code, &get_hardcoded_secret_config())
            .map_err(|e| e.to_string())?;

        assert_that(&rewritten.hardcoded_secret_result.is_some());
        assert_that(&rewritten.hardcoded_secret_result.unwrap().literals.len()).is_equal_to(0);

        Ok(())
    }

    #[test]
    fn test_require_literals_with_no_literal_discarded() -> Result<(), String> {
        let original_code = "const a = require(getmodule())".to_string();

        let rewritten = rewrite_js_with_config(original_code, &get_hardcoded_secret_config())
            .map_err(|e| e.to_string())?;

        assert_that(&rewritten.hardcoded_secret_result.is_some());
        assert_that(&rewritten.hardcoded_secret_result.unwrap().literals.len()).is_equal_to(0);

        Ok(())
    }

    #[test]
    fn test_require_literals_with_no_literal_spread_discarded() -> Result<(), String> {
        let original_code = "const a = require(...a)".to_string();

        let rewritten = rewrite_js_with_config(original_code, &get_hardcoded_secret_config())
            .map_err(|e| e.to_string())?;

        assert_that(&rewritten.hardcoded_secret_result.is_some());
        assert_that(&rewritten.hardcoded_secret_result.unwrap().literals.len()).is_equal_to(0);

        Ok(())
    }

    #[test]
    fn test_non_require_calls_literals_included() -> Result<(), String> {
        let original_code = "const a = no_require('literal_literal')".to_string();

        let rewritten = rewrite_js_with_config(original_code, &get_hardcoded_secret_config())
            .map_err(|e| e.to_string())?;

        assert_that(&rewritten.hardcoded_secret_result.is_some());
        assert_that(&rewritten.hardcoded_secret_result.unwrap().literals.len()).is_equal_to(1);

        Ok(())
    }

    #[test]
    fn test_new_regexp_is_discarded() -> Result<(), String> {
        let original_code = "const a = new RegExp('literal_literal')".to_string();

        let rewritten = rewrite_js_with_config(original_code, &get_hardcoded_secret_config())
            .map_err(|e| e.to_string())?;

        assert_that(&rewritten.hardcoded_secret_result.is_some());
        assert_that(&rewritten.hardcoded_secret_result.unwrap().literals.len()).is_equal_to(0);

        Ok(())
    }

    #[test]
    fn test_regexp_is_discarded() -> Result<(), String> {
        let original_code = "const a = /literal_literal/".to_string();

        let rewritten = rewrite_js_with_config(original_code, &get_hardcoded_secret_config())
            .map_err(|e| e.to_string())?;

        assert_that(&rewritten.hardcoded_secret_result.is_some());
        assert_that(&rewritten.hardcoded_secret_result.unwrap().literals.len()).is_equal_to(0);

        Ok(())
    }
}
