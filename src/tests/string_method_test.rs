/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/

#[cfg(test)]
mod tests {

    use crate::{
        tests::{rewrite_js, rewrite_js_with_exclusions, set_local_var},
        visitor::csi_methods::CsiExclusions,
    };
    use spectral::{assert_that, string::StrAssertions};

    #[cfg(test)]
    #[ctor::ctor]
    fn init() {
        set_local_var();
    }

    #[test]
    fn test_ident_substring() -> Result<(), String> {
        let original_code = "{const a = b.substring(1);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0, __datadog_test_1;
    const a = (__datadog_test_0 = b, __datadog_test_1 = __datadog_test_0.substring, _ddiast.string_substring(__datadog_test_1.call(__datadog_test_0, 1), __datadog_test_1, __datadog_test_0, 1));");
        Ok(())
    }

    #[test]
    fn test_call_substring() -> Result<(), String> {
        let original_code = "{const a = b().substring(1);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0, __datadog_test_1;
    const a = (__datadog_test_0 = b(), __datadog_test_1 = __datadog_test_0.substring, _ddiast.string_substring(__datadog_test_1.call(__datadog_test_0, 1), __datadog_test_1, __datadog_test_0, 1));");
        Ok(())
    }

    #[test]
    fn test_ident_substring_with_call_arg() -> Result<(), String> {
        let original_code = "{const a = b.substring(c());}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0, __datadog_test_1, __datadog_test_2;
    const a = (__datadog_test_0 = b, __datadog_test_1 = __datadog_test_0.substring, __datadog_test_2 = c(), _ddiast.string_substring(__datadog_test_1.call(__datadog_test_0, __datadog_test_2), __datadog_test_1, __datadog_test_0, __datadog_test_2));");
        Ok(())
    }

    #[test]
    fn test_call_substring_with_call_arg() -> Result<(), String> {
        let original_code = "{const a = b().substring(c());}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0, __datadog_test_1, __datadog_test_2;
    const a = (__datadog_test_0 = b(), __datadog_test_1 = __datadog_test_0.substring, __datadog_test_2 = c(), _ddiast.string_substring(__datadog_test_1.call(__datadog_test_0, __datadog_test_2), __datadog_test_1, __datadog_test_0, __datadog_test_2));");
        Ok(())
    }

    #[test]
    fn test_lit_substring() -> Result<(), String> {
        let original_code = "{const a = \"b\".substring(1);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("const a = \"b\".substring(1)");
        Ok(())
    }

    #[test]
    fn test_prototype_substring_with_literal_arg() -> Result<(), String> {
        let original_code = "{const a = String.prototype.substring.call('hello', 2);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("const a = String.prototype.substring.call('hello', 2)");
        Ok(())
    }

    #[test]
    fn test_prototype_substring_call_with_variable_arg() -> Result<(), String> {
        let original_code = "{const a = String.prototype.substring.call(b, 2);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("let __datadog_test_0, __datadog_test_1;
    const a = (__datadog_test_0 = b, __datadog_test_1 = __datadog_test_0.substring, _ddiast.string_substring(__datadog_test_1.call(__datadog_test_0, 2), __datadog_test_1, __datadog_test_0, 2));");
        Ok(())
    }

    #[test]
    fn test_prototype_substring_apply_with_variable_arg() -> Result<(), String> {
        let original_code = "{const a = String.prototype.substring.apply(b, [2]);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("let __datadog_test_0, __datadog_test_1;
    const a = (__datadog_test_0 = b, __datadog_test_1 = __datadog_test_0.substring, _ddiast.string_substring(__datadog_test_1.call(__datadog_test_0, 2), __datadog_test_1, __datadog_test_0, 2));");
        Ok(())
    }

    #[test]
    fn test_prototype_substring_apply_with_call_arg() -> Result<(), String> {
        let original_code = "{const a = String.prototype.substring.apply(b(), [2]);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("let __datadog_test_0, __datadog_test_1;
    const a = (__datadog_test_0 = b(), __datadog_test_1 = __datadog_test_0.substring, _ddiast.string_substring(__datadog_test_1.call(__datadog_test_0, 2), __datadog_test_1, __datadog_test_0, 2));");
        Ok(())
    }

    #[test]
    fn test_ident_trim() -> Result<(), String> {
        let original_code = "{const a = b.trim();}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0, __datadog_test_1;
    const a = (__datadog_test_0 = b, __datadog_test_1 = __datadog_test_0.trim, _ddiast.string_trim(__datadog_test_1.call(__datadog_test_0), __datadog_test_1, __datadog_test_0));");
        Ok(())
    }

    #[test]
    fn test_literal_trim() -> Result<(), String> {
        let original_code = "{const a = \"b\".trim();}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("const a = \"b\".trim();");
        Ok(())
    }

    #[test]
    fn test_csi_exclusion() -> Result<(), String> {
        let original_code = "{const a = b.concat('hello')}".to_string();
        let js_file = "test.js".to_string();
        let csi_exclusions = Some(vec!["String.prototype.concat".to_string()]);
        let rewritten = rewrite_js_with_exclusions(
            original_code,
            js_file,
            CsiExclusions::from(&csi_exclusions),
        )
        .map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("const a = b.concat('hello')");
        Ok(())
    }
}
