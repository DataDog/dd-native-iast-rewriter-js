/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/

#[cfg(test)]
mod tests {
    use spectral::{assert_that, string::StrAssertions};

    use crate::tests::{rewrite_js, set_local_var};

    #[cfg(test)]
    #[ctor::ctor]
    fn init() {
        set_local_var();
    }

    #[test]
    fn test_simple_assignation() -> Result<(), String> {
        let original_code = "{a += b;}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("a = (__datadog_test_0 = a, __datadog_test_1 = b, _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1))");
        Ok(())
    }

    #[test]
    fn test_plus_and_assignation() -> Result<(), String> {
        let original_code = "{a += b + c;}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("a = (__datadog_test_2 = a, __datadog_test_3 = (__datadog_test_0 = b, __datadog_test_1 = c, _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1)), _ddiast.plusOperator(__datadog_test_2 + __datadog_test_3, __datadog_test_2, __datadog_test_3))");
        Ok(())
    }

    #[test]
    fn test_call_assignation() -> Result<(), String> {
        let original_code = "for (let i = 0; i < buf.length; i++) {
            res1 += s.write(buf.slice(i, i + 1));
          }"
        .to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("for(let i = 0; i < buf.length; i++){
    let __datadog_test_0, __datadog_test_1, __datadog_test_2;
    res1 = (__datadog_test_1 = res1, __datadog_test_2 = s.write(buf.slice(i, (__datadog_test_0 = i, _ddiast.plusOperator(__datadog_test_0 + 1, __datadog_test_0, 1)))), _ddiast.plusOperator(__datadog_test_1 + __datadog_test_2, __datadog_test_1, __datadog_test_2));\n}");
        Ok(())
    }

    #[test]
    fn test_conditional_and_assignation() -> Result<(), String> {
        let original_code = "{a += b ? c : d;}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0, __datadog_test_1;
    a = (__datadog_test_0 = a, __datadog_test_1 = b ? c : d, _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1))");
        Ok(())
    }

    #[test]
    fn test_conditional_with_call_and_assignation() -> Result<(), String> {
        let original_code = "{a += b ? c() : d;}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0, __datadog_test_1;
    a = (__datadog_test_0 = a, __datadog_test_1 = b ? c() : d, _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1))");
        Ok(())
    }

    #[test]
    fn test_conditional_with_call_add_and_assignation() -> Result<(), String> {
        let original_code = "{a += b ? c(e() + f) : d;}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3;
    a = (__datadog_test_2 = a, __datadog_test_3 = b ? c((__datadog_test_0 = e(), __datadog_test_1 = f, _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1))) : d, _ddiast.plusOperator(__datadog_test_2 + __datadog_test_3, __datadog_test_2, __datadog_test_3))");
        Ok(())
    }
}
