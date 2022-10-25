/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/

#[cfg(test)]
mod tests {

    use crate::tests::{rewrite_js, set_local_var};
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
            .contains("let __datadog_test_0;
    const a = (__datadog_test_0 = b, _ddiast.string_substring(__datadog_test_0.substring(1), __datadog_test_0, 1));");
        Ok(())
    }

    #[test]
    fn test_call_substring() -> Result<(), String> {
        let original_code = "{const a = b().substring(1);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0;
    const a = (__datadog_test_0 = b(), _ddiast.string_substring(__datadog_test_0.substring(1), __datadog_test_0, 1));");
        Ok(())
    }

    #[test]
    fn test_ident_substring_with_call_arg() -> Result<(), String> {
        let original_code = "{const a = b.substring(c());}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0, __datadog_test_1;
    const a = (__datadog_test_0 = b, __datadog_test_1 = c(), _ddiast.string_substring(__datadog_test_0.substring(__datadog_test_1), __datadog_test_0, __datadog_test_1));");
        Ok(())
    }

    #[test]
    fn test_call_substring_with_call_arg() -> Result<(), String> {
        let original_code = "{const a = b().substring(c());}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0, __datadog_test_1;
    const a = (__datadog_test_0 = b(), __datadog_test_1 = c(), _ddiast.string_substring(__datadog_test_0.substring(__datadog_test_1), __datadog_test_0, __datadog_test_1));");
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
        assert_that(&rewritten.code).contains("let __datadog_test_0;
    const a = (__datadog_test_0 = b, _ddiast.string_substring(__datadog_test_0.substring(2), __datadog_test_0, 2));");
        Ok(())
    }

    #[test]
    fn test_prototype_substring_apply_with_variable_arg() -> Result<(), String> {
        let original_code = "{const a = String.prototype.substring.apply(b, [2]);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("let __datadog_test_0;
    const a = (__datadog_test_0 = b, _ddiast.string_substring(__datadog_test_0.substring(2), __datadog_test_0, 2));");
        Ok(())
    }

    #[test]
    fn test_prototype_substring_apply_with_call_arg() -> Result<(), String> {
        let original_code = "{const a = String.prototype.substring.apply(b(), [2]);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("let __datadog_test_0;
    const a = (__datadog_test_0 = b(), _ddiast.string_substring(__datadog_test_0.substring(2), __datadog_test_0, 2));");
        Ok(())
    }
}
