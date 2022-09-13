/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/

#[cfg(test)]
mod tests {
    use spectral::{assert_that, string::StrAssertions};

    use crate::{rewrite_js, tests::set_local_var};

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
        assert_that(&rewritten.code).contains("a = global._ddiast.plusOperator(a + b, a, b)");
        Ok(())
    }

    #[test]
    fn test_plus_and_assignation() -> Result<(), String> {
        let original_code = "{a += b + c;}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("a = global._ddiast.plusOperator(a + b + c, a, b, c)");
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
    let __datadog_test_0;
    res1 = (__datadog_test_0 = s.write(buf.slice(i, global._ddiast.plusOperator(i + 1, i, 1))), global._ddiast.plusOperator(res1 + __datadog_test_0, res1, __datadog_test_0));\n}");
        Ok(())
    }
}
