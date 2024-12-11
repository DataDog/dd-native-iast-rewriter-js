/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/

#[cfg(test)]
mod tests {

    use crate::tests::rewrite_js;
    use speculoos::{assert_that, string::StrAssertions};

    #[test]
    fn test_template_literal() -> Result<(), String> {
        let original_code = "{const a = `${b}Hello`}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("const a = (__datadog_test_0 = b, _ddiast.tplOperator(`${__datadog_test_0}Hello`, __datadog_test_0));");
        Ok(())
    }

    #[test]
    fn test_template_literal_ending_expr() -> Result<(), String> {
        let original_code = "{const a = `Hello${b}`}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("const a = (__datadog_test_0 = b, _ddiast.tplOperator(`Hello${__datadog_test_0}`, __datadog_test_0));");
        Ok(())
    }

    #[test]
    fn test_template_literal_with_call() -> Result<(), String> {
        let original_code = "{const a = `He${b()}llo`}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0;
    const a = (__datadog_test_0 = b(), _ddiast.tplOperator(`He${__datadog_test_0}llo`, __datadog_test_0));");
        Ok(())
    }

    #[test]
    fn test_template_literal_with_binary() -> Result<(), String> {
        let original_code = "{const a = `He${b + c}llo`}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0;
    const a = (__datadog_test_0 = _ddiast.plusOperator(b + c, b, c), _ddiast.tplOperator(`He${__datadog_test_0}llo`, __datadog_test_0));");
        Ok(())
    }

    #[test]
    fn test_template_literal_with_binary_and_call() -> Result<(), String> {
        let original_code = "{const a = `He${b + c()}llo`}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0, __datadog_test_1, __datadog_test_2;
    const a = (__datadog_test_2 = (__datadog_test_0 = b, __datadog_test_1 = c(), _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1)), _ddiast.tplOperator(`He${__datadog_test_2}llo`, __datadog_test_2));");
        Ok(())
    }

    #[test]
    fn test_template_literal_with_typeof_and_more_is_modified() -> Result<(), String> {
        let original_code = "{const a = `He${typeof b}llo wor${a}ld`}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("const a = (__datadog_test_0 = typeof b, __datadog_test_1 = a, _ddiast.tplOperator(`He${__datadog_test_0}llo wor${__datadog_test_1}ld`, __datadog_test_0, __datadog_test_1));");
        Ok(())
    }

    #[test]
    fn test_template_literal_with_property_access() -> Result<(), String> {
        let original_code = "{const a = `Hello world ${b.x}`}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0;\n    const a = (__datadog_test_0 = b.x, _ddiast.tplOperator(`Hello world ${__datadog_test_0}`, __datadog_test_0));");
        Ok(())
    }

    #[test]
    fn test_template_literal_with_yield() -> Result<(), String> {
        let original_code = "function* foo() {
            var f = `foo${ yield 'yielded' }bar`;
            return f;
        }"
        .to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("function* foo() {\n    let __datadog_test_0;
    var f = (__datadog_test_0 = yield 'yielded', _ddiast.tplOperator(`foo${__datadog_test_0}bar`, __datadog_test_0));");
        Ok(())
    }

    #[test]
    fn test_template_literal_with_arrow() -> Result<(), String> {
        let original_code = "function names(arg) {
            const flag = arg;
            const addPrefix = (value) => (flag ? value : `\"my_prefix_${value}\"`);
            const result = `${addPrefix('NAME_0')}`;
            return result;
    }"
        .to_string();

        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;

        assert_that(&rewritten.code).contains("let __datadog_test_0, __datadog_test_1;");
        Ok(())
    }
}
