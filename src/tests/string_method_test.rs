/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/

#[cfg(test)]
mod tests {

    use crate::{
        rewriter::print_js,
        tests::{
            csi_from_str, get_chained_and_print_comments_config, get_default_config, rewrite_js,
            rewrite_js_with_csi_methods,
        },
        visitor::csi_methods::CsiMethods,
    };
    use speculoos::{assert_that, string::StrAssertions};

    #[test]
    fn test_ident_substring() -> Result<(), String> {
        let original_code = "{const a = b.substring(1);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0, __datadog_test_1;
    const a = (__datadog_test_0 = b, __datadog_test_1 = __datadog_test_0.substring, _ddiast.stringSubstring(__datadog_test_1.call(__datadog_test_0, 1), __datadog_test_1, __datadog_test_0, 1));");
        Ok(())
    }

    #[test]
    fn test_call_substring() -> Result<(), String> {
        let original_code = "{const a = b().substring(1);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0, __datadog_test_1;
    const a = (__datadog_test_0 = b(), __datadog_test_1 = __datadog_test_0.substring, _ddiast.stringSubstring(__datadog_test_1.call(__datadog_test_0, 1), __datadog_test_1, __datadog_test_0, 1));");
        Ok(())
    }

    #[test]
    fn test_ident_substring_with_call_arg() -> Result<(), String> {
        let original_code = "{const a = b.substring(c());}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0, __datadog_test_1, __datadog_test_2;
    const a = (__datadog_test_0 = b, __datadog_test_1 = __datadog_test_0.substring, __datadog_test_2 = c(), _ddiast.stringSubstring(__datadog_test_1.call(__datadog_test_0, __datadog_test_2), __datadog_test_1, __datadog_test_0, __datadog_test_2));");
        Ok(())
    }

    #[test]
    fn test_call_substring_with_call_arg() -> Result<(), String> {
        let original_code = "{const a = b().substring(c());}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0, __datadog_test_1, __datadog_test_2;
    const a = (__datadog_test_0 = b(), __datadog_test_1 = __datadog_test_0.substring, __datadog_test_2 = c(), _ddiast.stringSubstring(__datadog_test_1.call(__datadog_test_0, __datadog_test_2), __datadog_test_1, __datadog_test_0, __datadog_test_2));");
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
    const a = (__datadog_test_0 = b, __datadog_test_1 = String.prototype.substring, _ddiast.stringSubstring(__datadog_test_1.call(__datadog_test_0, 2), __datadog_test_1, __datadog_test_0, 2));");
        Ok(())
    }

    #[test]
    fn test_prototype_substring_apply_with_variable_arg() -> Result<(), String> {
        let original_code = "{const a = String.prototype.substring.apply(b, [2]);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("let __datadog_test_0, __datadog_test_1;
    const a = (__datadog_test_0 = b, __datadog_test_1 = String.prototype.substring, _ddiast.stringSubstring(__datadog_test_1.apply(__datadog_test_0, [\n        2\n    ]), __datadog_test_1, __datadog_test_0, 2));");
        Ok(())
    }

    #[test]
    fn test_prototype_substring_apply_with_call_arg() -> Result<(), String> {
        let original_code = "{const a = String.prototype.substring.apply(b(), [c]);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("let __datadog_test_0, __datadog_test_1, __datadog_test_2;
    const a = (__datadog_test_0 = b(), __datadog_test_1 = String.prototype.substring, __datadog_test_2 = c, _ddiast.stringSubstring(__datadog_test_1.apply(__datadog_test_0, [\n        __datadog_test_2\n    ]), __datadog_test_1, __datadog_test_0, __datadog_test_2));");
        Ok(())
    }

    #[test]
    fn test_prototype_concat_call() -> Result<(), String> {
        let original_code = "{const a = String.prototype.concat.call(1,a(),3)}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0, __datadog_test_1;
    const a = (__datadog_test_0 = String.prototype.concat, __datadog_test_1 = a(), _ddiast.stringConcat(__datadog_test_0.call(1, __datadog_test_1, 3), __datadog_test_0, 1, __datadog_test_1, 3));");
        Ok(())
    }

    #[test]
    fn test_prototype_concat_call_literals() -> Result<(), String> {
        let original_code = "{const a = String.prototype.concat.call(1,2,3)}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("const a = String.prototype.concat.call(1,2,3)");
        Ok(())
    }

    #[test]
    fn test_prototype_concat_call_this_spread() -> Result<(), String> {
        let original_code = "{const c = String.prototype.concat.call(...a)}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("const c = (__datadog_test_0 = String.prototype.concat, __datadog_test_1 = [\n        ...a\n    ], _ddiast.stringConcat(__datadog_test_0.call(...__datadog_test_1), __datadog_test_0, ...__datadog_test_1));");
        Ok(())
    }

    #[test]
    fn test_prototype_concat_call_args_spread() -> Result<(), String> {
        let original_code = "{const c = String.prototype.concat.call(a, ...b)}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("const c = (__datadog_test_0 = a, __datadog_test_1 = String.prototype.concat, __datadog_test_2 = [\n        ...b\n    ], _ddiast.stringConcat(__datadog_test_1.call(__datadog_test_0, ...__datadog_test_2), __datadog_test_1, __datadog_test_0, ...__datadog_test_2));");
        Ok(())
    }

    #[test]
    fn test_prototype_concat_apply_args_spread() -> Result<(), String> {
        let original_code = "{const c = String.prototype.concat.apply(a, ...b)}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("const c = (__datadog_test_0 = a, __datadog_test_1 = String.prototype.concat, __datadog_test_2 = [\n        ...b\n    ], _ddiast.stringConcat(__datadog_test_1.apply(__datadog_test_0, ...__datadog_test_2), __datadog_test_1, __datadog_test_0, ...__datadog_test_2));");
        Ok(())
    }

    #[test]
    fn test_concat_args_spread() -> Result<(), String> {
        let original_code = "{const c = a.concat(...b)}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("const c = (__datadog_test_0 = a, __datadog_test_1 = __datadog_test_0.concat, __datadog_test_2 = [\n        ...b\n    ], _ddiast.stringConcat(__datadog_test_1.call(__datadog_test_0, ...__datadog_test_2), __datadog_test_1, __datadog_test_0, ...__datadog_test_2));");
        Ok(())
    }

    #[test]
    fn test_prototype_concat_call_args_spread_multiple() -> Result<(), String> {
        let original_code = "{const c = String.prototype.concat.call(a, ...b, ...c)}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("const c = (__datadog_test_0 = a, __datadog_test_1 = String.prototype.concat, __datadog_test_2 = [\n        ...b\n    ], __datadog_test_3 = [\n        ...c\n    ], _ddiast.stringConcat(__datadog_test_1.call(__datadog_test_0, ...__datadog_test_2, ...__datadog_test_3), __datadog_test_1, __datadog_test_0, ...__datadog_test_2, ...__datadog_test_3));");
        Ok(())
    }

    #[test]
    fn test_ident_trim() -> Result<(), String> {
        let original_code = "{const a = b.trim();}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0, __datadog_test_1;
    const a = (__datadog_test_0 = b, __datadog_test_1 = __datadog_test_0.trim, _ddiast.stringTrim(__datadog_test_1.call(__datadog_test_0), __datadog_test_1, __datadog_test_0));");
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
    fn test_chained_calls() -> Result<(), String> {
        let original_code = "{const a = b.concat('a').substring(2).trim();}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("let __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3, __datadog_test_4, __datadog_test_5;
    const a = (__datadog_test_4 = (__datadog_test_2 = (__datadog_test_0 = b, __datadog_test_1 = __datadog_test_0.concat, _ddiast.stringConcat(__datadog_test_1.call(__datadog_test_0, 'a'), __datadog_test_1, __datadog_test_0, 'a')), __datadog_test_3 = __datadog_test_2.substring, _ddiast.stringSubstring(__datadog_test_3.call(__datadog_test_2, 2), __datadog_test_3, __datadog_test_2, 2)), __datadog_test_5 = __datadog_test_4.trim, _ddiast.stringTrim(__datadog_test_5.call(__datadog_test_4), __datadog_test_5, __datadog_test_4));");
        Ok(())
    }

    #[test]
    fn test_chained_calls_with_exclusions() -> Result<(), String> {
        let original_code = "{const a = b.concat('a').substring(2).trim();}".to_string();
        let js_file = "test.js".to_string();
        let mut methods = vec![csi_from_str("concat", Some("stringConcat"))];
        let rewritten =
            rewrite_js_with_csi_methods(original_code, js_file, &CsiMethods::new(&mut methods))
                .map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("let __datadog_test_0, __datadog_test_1;
    const a = (__datadog_test_0 = b, __datadog_test_1 = __datadog_test_0.concat, _ddiast.stringConcat(__datadog_test_1.call(__datadog_test_0, 'a'), __datadog_test_1, __datadog_test_0, 'a')).substring(2).trim();");
        Ok(())
    }

    #[test]
    fn test_csi_exclusion() -> Result<(), String> {
        let original_code = "{const a = b.concat('hello')}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js_with_csi_methods(original_code, js_file, &CsiMethods::empty())
            .map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("const a = b.concat('hello')");
        Ok(())
    }

    #[test]
    fn test_plus_operator_exclusion_by_default() -> Result<(), String> {
        let original_code = "{const a = b.plusOperator('hello')}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("const a = b.plusOperator('hello')");
        Ok(())
    }

    #[test]
    fn test_plus_operator_csi_method_but_plus_exclusion() -> Result<(), String> {
        let original_code = "{const a = b.plusOperator(c + d)}".to_string();
        let js_file = "test.js".to_string();
        let mut methods = vec![csi_from_str("plusOperator", Some("plusOperator"))];
        let rewritten =
            rewrite_js_with_csi_methods(original_code, js_file, &CsiMethods::new(&mut methods))
                .map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("let __datadog_test_0, __datadog_test_1;
    const a = (__datadog_test_0 = b, __datadog_test_1 = __datadog_test_0.plusOperator, _ddiast.plusOperator(__datadog_test_1.call(__datadog_test_0, c + d), __datadog_test_1, __datadog_test_0));");
        Ok(())
    }

    #[test]
    fn test_array_and_join() -> Result<(), String> {
        let original_code = "{[str, str].join();}".to_string();
        let js_file = "test.js".to_string();
        let mut methods = vec![csi_from_str("join", None)];
        let rewritten =
            rewrite_js_with_csi_methods(original_code, js_file, &CsiMethods::new(&mut methods))
                .map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("let __datadog_test_0, __datadog_test_1;
    (__datadog_test_0 = [
        str,
        str
    ], __datadog_test_1 = __datadog_test_0.join, _ddiast.join(__datadog_test_1.call(__datadog_test_0), __datadog_test_1, __datadog_test_0));");
        Ok(())
    }

    #[test]
    fn test_replace_with_sourcemapping() -> Result<(), String> {
        let original_code =
            "{const a = { __html: b.replace(/\\/\\*# sourceMappingURL=.*\\*\\//g, '') + 'other' }}"
                .to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file)
            .map_err(|e| e.to_string())
            .map(|rewrite_output| print_js(&rewrite_output, &get_default_config(false)))?;

        assert_that(&rewritten).contains("const a = {\n        __html: (__datadog_test_2 = (__datadog_test_0 = b, \
    __datadog_test_1 = __datadog_test_0.replace, _ddiast.replace(__datadog_test_1.call(__datadog_test_0, /\\/\\*# sourceMappingURL=.*\\*\\//g, ''), \
    __datadog_test_1, __datadog_test_0, /\\/\\*# sourceMappingURL=.*\\*\\//g, '')), _ddiast.plusOperator(__datadog_test_2 + 'other', __datadog_test_2, 'other'))
    }");
        Ok(())
    }

    #[test]
    fn test_replace_with_sourcemapping_url() -> Result<(), String> {
        let original_code =
            "{const a = { __html: b.replace(/\\/\\*# sourceMappingURL=.*\\*\\//g, '') + 'other' }}
//# sourceMappingURL=StrUtil.js.map
        "
            .to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file)
            .map_err(|e| e.to_string())
            .map(|rewrite_output| {
                print_js(&rewrite_output, &get_chained_and_print_comments_config())
            })?;

        assert_that(&rewritten).contains("const a = {\n        __html: (__datadog_test_2 = (__datadog_test_0 = b, \
    __datadog_test_1 = __datadog_test_0.replace, _ddiast.replace(__datadog_test_1.call(__datadog_test_0, /\\/\\*# sourceMappingURL=.*\\*\\//g, ''), \
    __datadog_test_1, __datadog_test_0, /\\/\\*# sourceMappingURL=.*\\*\\//g, '')), _ddiast.plusOperator(__datadog_test_2 + 'other', __datadog_test_2, 'other'))
    }");
        Ok(())
    }

    #[test]
    fn test_replace_with_sourcemapping_url_embebbed() -> Result<(), String> {
        let original_code = "{const a = { __html: b.replace(/\\/\\*# sourceMappingURL=.*\\*\\//g, '') + 'other' }}
        //# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiU3RyVXRpbC5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIlN0clV0aWwudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6Ijs7QUFBQTtJQUFBO0lBZ0JBLENBQUM7SUFmQyxxQkFBRyxHQUFILFVBQUksQ0FBUztRQUNYLE9BQU8sR0FBRyxHQUFHLENBQUMsQ0FBQTtJQUNoQixDQUFDO0lBRU0sd0JBQU0sR0FBYixVQUFjLENBQVMsRUFBRSxDQUFTO1FBQ2hDLE9BQU8sQ0FBQyxHQUFHLElBQUksQ0FBQyxHQUFHLENBQUMsSUFBSSxDQUFDLEtBQUssQ0FBQyxDQUFDLENBQUMsQ0FBQyxDQUFBO0lBQ3BDLENBQUM7SUFFTSx1QkFBSyxHQUFaLFVBQWEsQ0FBUztRQUNwQixPQUFPLENBQUMsQ0FBQyxRQUFRLEVBQUUsQ0FBQTtJQUNyQixDQUFDO0lBRU0scUJBQUcsR0FBVixVQUFXLENBQVMsRUFBRSxDQUFTO1FBQzdCLE9BQU8sQ0FBQyxHQUFHLENBQUMsQ0FBQTtJQUNkLENBQUM7SUFDSCxjQUFDO0FBQUQsQ0FBQyxBQWhCRCxJQWdCQyJ9
        ".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file)
            .map_err(|e| e.to_string())
            .map(|rewrite_output| print_js(&rewrite_output, &get_default_config(false)))?;

        assert_that(&rewritten).contains("const a = {\n        __html: (__datadog_test_2 = (__datadog_test_0 = b, \
    __datadog_test_1 = __datadog_test_0.replace, _ddiast.replace(__datadog_test_1.call(__datadog_test_0, /\\/\\*# sourceMappingURL=.*\\*\\//g, ''), \
    __datadog_test_1, __datadog_test_0, /\\/\\*# sourceMappingURL=.*\\*\\//g, '')), _ddiast.plusOperator(__datadog_test_2 + 'other', __datadog_test_2, 'other'))
    }");
        Ok(())
    }
}
