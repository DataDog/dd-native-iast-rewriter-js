/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/

#[cfg(test)]
mod tests {

    use crate::{tests::rewrite_js, transform::transform_status::Status};
    use speculoos::{assert_that, string::StrAssertions};

    #[test]
    fn test_arrow_simple_not_modified() -> Result<(), String> {
        let original_code = "{const a = (arg) => arg ? true : false}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;

        assert_that(
            &rewritten
                .transform_status
                .is_some_and(|status| status.status == Status::NotModified),
        );

        Ok(())
    }

    #[test]
    fn test_arrow_paren_not_modified() -> Result<(), String> {
        let original_code = "{const a = (arg) => (arg ? true : false)}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;

        assert_that(
            &rewritten
                .transform_status
                .is_some_and(|status| status.status == Status::NotModified),
        );

        Ok(())
    }

    #[test]
    fn test_arrow_block_not_modified() -> Result<(), String> {
        let original_code = "{const a = (arg) => { return arg ? true : false }}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;

        assert_that(
            &rewritten
                .transform_status
                .is_some_and(|status| status.status == Status::NotModified),
        );

        Ok(())
    }

    #[test]
    fn test_arrow_object_not_modified() -> Result<(), String> {
        let original_code = "{const a = (arg) => ({ test: true }) }".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;

        assert_that(
            &rewritten
                .transform_status
                .is_some_and(|status| status.status == Status::NotModified),
        );

        Ok(())
    }

    #[test]
    fn test_arrow_object_2_not_modified() -> Result<(), String> {
        let original_code = "{const a = (arg) => { test: true } }".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;

        assert_that(
            &rewritten
                .transform_status
                .is_some_and(|status| status.status == Status::NotModified),
        );

        Ok(())
    }

    #[test]
    fn test_arrow_modified() -> Result<(), String> {
        let original_code = "{const a = (arg) => arg ? `hello ${arg}` : '' }".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;

        assert_that(
            &rewritten
                .transform_status
                .is_some_and(|status| status.status == Status::Modified),
        );
        assert_that(&rewritten.code).contains("const a = (arg)=>{
        let __datadog_test_0;
        return arg ? (__datadog_test_0 = arg, _ddiast.tplOperator(`hello ${__datadog_test_0}`, __datadog_test_0)) : '';
    };");

        Ok(())
    }
}
