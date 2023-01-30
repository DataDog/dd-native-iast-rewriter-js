/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/

#[cfg(test)]
mod tests {
    use spectral::{assert_that, string::StrAssertions};

    use crate::tests::rewrite_js;

    #[test]
    fn test_sourcemap_span() -> Result<(), String> {
        let original_code = "{const a = b.substring(1);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
          .contains("let __datadog_test_0, __datadog_test_1;
  const a = (__datadog_test_0 = b, __datadog_test_1 = __datadog_test_0.substring, _ddiast.stringSubstring(__datadog_test_1.call(__datadog_test_0, 1), __datadog_test_1, __datadog_test_0, 1));");
        Ok(())
    }
}
