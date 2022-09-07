/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/

#[cfg(test)]
mod tests {

    use crate::{rewriter::rewrite_js, tests::set_local_var};
    use spectral::{assert_that, string::StrAssertions};

    #[cfg(test)]
    #[ctor::ctor]
    fn init() {
        set_local_var();
    }

    #[test]
    fn test_template_literal() -> Result<(), String> {
        let original_code = "{const a = `${b}Hello`}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("const a = global._ddiast.plusOperator(`${b}Hello`, b, `Hello`);");
        Ok(())
    }

    #[test]
    fn test_template_literal_with_call() -> Result<(), String> {
        let original_code = "{const a = `He${b()}llo`}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0;\n    const a = (__datadog_test_0 = b(), global._ddiast.plusOperator(`He${__datadog_test_0}llo`, `He`, __datadog_test_0, `llo`))");
        Ok(())
    }

    #[test]
    fn test_template_literal_with_binary() -> Result<(), String> {
        let original_code = "{const a = `He${b + c}llo`}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0;\n    const a = (__datadog_test_0 = global._ddiast.plusOperator(b + c, b, c), global._ddiast.plusOperator(`He${__datadog_test_0}llo`, `He`, __datadog_test_0, `llo`))");
        Ok(())
    }

    #[test]
    fn test_template_literal_with_binary_and_call() -> Result<(), String> {
        let original_code = "{const a = `He${b + c()}llo`}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0, __datadog_test_1;\n    const a = (__datadog_test_1 = (__datadog_test_0 = c(), global._ddiast.plusOperator(b + __datadog_test_0, b, __datadog_test_0)), global._ddiast.plusOperator(`He${__datadog_test_1}llo`, `He`, __datadog_test_1, `llo`))");
        Ok(())
    }
}
