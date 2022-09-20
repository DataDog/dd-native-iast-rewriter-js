/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/

#[cfg(test)]
mod tests {

    use spectral::{assert_that, prelude::ContainingIntoIterAssertions, string::StrAssertions};

    use anyhow::Error;

    use crate::{
        rewriter::debug_js,
        tests::{rewrite_js, set_local_var},
    };

    #[cfg(test)]
    #[ctor::ctor]
    fn init() {
        set_local_var();
    }

    #[test]
    #[ignore]
    #[allow(unused_must_use)]
    fn test_paren_stmt() -> Result<(), Error> {
        let original_code = "(a,b,c)".to_string();
        debug_js(original_code);
        Ok(())
    }

    #[test]
    fn test_class_multiple_block() -> Result<(), String> {
        let original_code = "
        class Polygon {
            constructor(height, width) {
              this.height = height;
              this.width = width;
              if (this.height > 100) {
                this.greaterThan100 = true;
              }
            }
            sum() { return this.height + this.width };
          }
        "
        .to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains(
            "return (__datadog_test_0 = this.height, __datadog_test_1 = this.width, _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1));",
        );
        Ok(())
    }

    #[test]
    fn test_class_multiple_block_binary_and_call() -> Result<(), String> {
        let original_code = "
        class Polygon {
            constructor(height, width) {
              this.height = height;
              this.width = width;
              if (this.height > 100) {
                this.greaterThan100 = true;
              }
            }
            w() {return this.width;}
            sum() { return this.height + this.w(); }
          }
        "
        .to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("(__datadog_test_0 = this.height, __datadog_test_1 = this.w(), _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1))");
        Ok(())
    }

    #[test]
    fn test_simple_plus_literal() -> Result<(), String> {
        let original_code = "{const result = 'a' + 'b'}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("const result = 'a' + 'b'");
        Ok(())
    }

    #[test]
    fn test_simple_3_plus_literal() -> Result<(), String> {
        let original_code = "{const result = 'a' + 'b' + 'c'}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("const result = 'a' + 'b' + 'c'");
        Ok(())
    }

    #[test]
    fn test_variable_plus_literals() -> Result<(), String> {
        let original_code = "{const result = a + 'b' + 'c'}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("const result = (__datadog_test_1 = (__datadog_test_0 = a, _ddiast.plusOperator(__datadog_test_0 + 'b', __datadog_test_0, 'b')), _ddiast.plusOperator(__datadog_test_1 + 'c', __datadog_test_1, 'c'))");
        Ok(())
    }

    #[test]
    fn test_simple_plus_smi() -> Result<(), String> {
        let original_code = "{const result = 1 + 2}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("const result = 1 + 2");
        Ok(())
    }

    #[test]
    fn test_simple_plus() -> Result<(), String> {
        let original_code = "{const result = a + b}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("let __datadog_test_0, __datadog_test_1;
    const result = (__datadog_test_0 = a, __datadog_test_1 = b, _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1))");
        Ok(())
    }

    #[test]
    fn test_simple_plus_call() -> Result<(), String> {
        let original_code = "{const result = a + b()}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0, __datadog_test_1;
    const result = (__datadog_test_0 = a, __datadog_test_1 = b(), _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1))");
        Ok(())
    }

    #[test]
    fn test_multiple_plus() -> Result<(), String> {
        let original_code = "{const result = a + b + c + d}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("let __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3, __datadog_test_4, __datadog_test_5;
    const result = (__datadog_test_4 = (__datadog_test_2 = (__datadog_test_0 = a, __datadog_test_1 = b, _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1)), __datadog_test_3 = c, _ddiast.plusOperator(__datadog_test_2 + __datadog_test_3, __datadog_test_2, __datadog_test_3)), __datadog_test_5 = d, _ddiast.plusOperator(__datadog_test_4 + __datadog_test_5, __datadog_test_4, __datadog_test_5))");
        Ok(())
    }

    #[test]
    fn test_multiple_plus_and_func() -> Result<(), String> {
        let original_code = "{const result = a + b + c() + d}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("let __datadog_test_0, __datadog_test_1, __datadog_test_2, __datadog_test_3, __datadog_test_4, __datadog_test_5;
    const result = (__datadog_test_4 = (__datadog_test_2 = (__datadog_test_0 = a, __datadog_test_1 = b, _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1)), __datadog_test_3 = c(), _ddiast.plusOperator(__datadog_test_2 + __datadog_test_3, __datadog_test_2, __datadog_test_3)), __datadog_test_5 = d, _ddiast.plusOperator(__datadog_test_4 + __datadog_test_5, __datadog_test_4, __datadog_test_5))");
        Ok(())
    }

    #[test]
    fn test_multiple_plus_and_inlined_func() -> Result<(), String> {
        let original_code =
            "{const result = a + b + (function(a){return a + 'epa';})(a)}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("const result = (__datadog_test_2 = (__datadog_test_0 = a, __datadog_test_1 = b, _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1)), __datadog_test_3 = (function(a) {
        let __datadog_test_0;\n        return (__datadog_test_0 = a, _ddiast.plusOperator(__datadog_test_0 + 'epa', __datadog_test_0, 'epa'));
    })(a), _ddiast.plusOperator(__datadog_test_2 + __datadog_test_3, __datadog_test_2, __datadog_test_3))");
        Ok(())
    }

    #[test]
    fn test_multiple_plus_inside_if_and_func() -> Result<(), String> {
        let original_code = "
        function fn1(a){
            return a + '.' + a;
        }
        function fn2(a){
            return a + '-' + a;
        }
        const fn = function(a, b){
            const c = a === 'hello' ? 'foo' + fn1(b) : 'bar' + fn2(b);
            return fn2(a) + c;
        }"
        .to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;

        assert_that(&rewritten.code)
            .contains("return (__datadog_test_1 = (__datadog_test_0 = a, _ddiast.plusOperator(__datadog_test_0 + '.', __datadog_test_0, '.')), __datadog_test_2 = a, _ddiast.plusOperator(__datadog_test_1 + __datadog_test_2, __datadog_test_1, __datadog_test_2))");

        assert_that(&rewritten.code)
            .contains("return (__datadog_test_1 = (__datadog_test_0 = a, _ddiast.plusOperator(__datadog_test_0 + '-', __datadog_test_0, '-')), __datadog_test_2 = a, _ddiast.plusOperator(__datadog_test_1 + __datadog_test_2, __datadog_test_1, __datadog_test_2))");

        assert_that(&rewritten.code)
            .contains("const c = a === 'hello' ? (__datadog_test_0 = fn1(b), _ddiast.plusOperator('foo' + __datadog_test_0, 'foo', __datadog_test_0)) : (__datadog_test_1 = fn2(b), _ddiast.plusOperator('bar' + __datadog_test_1, 'bar', __datadog_test_1));\n    return (__datadog_test_2 = fn2(a), __datadog_test_3 = c, _ddiast.plusOperator(__datadog_test_2 + __datadog_test_3, __datadog_test_2, __datadog_test_3))");
        Ok(())
    }

    #[test]
    fn test_simple_plus_with_multiply() -> Result<(), String> {
        let original_code = "{const result = a + b * c}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("const result = (__datadog_test_0 = a, __datadog_test_1 = b * c, _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1))");
        Ok(())
    }

    #[test]
    fn test_plus_and_block() -> Result<(), String> {
        let original_code = "{let b;const a = 'a' + (b = '_b_', b + c);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0, __datadog_test_1, __datadog_test_2;\n    let b;
    const a = (__datadog_test_2 = (b = '_b_', (__datadog_test_0 = b, __datadog_test_1 = c, _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1))), _ddiast.plusOperator('a' + __datadog_test_2, 'a', __datadog_test_2))");
        Ok(())
    }

    #[test]
    fn test_plus_inside_array() -> Result<(), String> {
        let original_code = "{const a = [a + b, c + d];}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("const a = [\n        (__datadog_test_0 = a, __datadog_test_1 = b, _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1)),
        (__datadog_test_2 = c, __datadog_test_3 = d, _ddiast.plusOperator(__datadog_test_2 + __datadog_test_3, __datadog_test_2, __datadog_test_3))\n    ]");
        Ok(())
    }

    #[test]
    fn test_plus_inside_object_assign() -> Result<(), String> {
        let original_code = "{const a = Object.assign({[prop]: a + b})}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains(
            "const a = Object.assign({
        [prop]: (__datadog_test_0 = a, __datadog_test_1 = b, _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1))",
        );
        Ok(())
    }

    #[test]
    fn test_match_declared_variables_same_block() -> Result<(), String> {
        let original_code = "{const __datadog_test_0 = 666; const c = a + b();}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string());
        assert!(&rewritten.is_err());
        assert_that!(&rewritten.err()).contains(
            "Cancelling test.js file rewrite. Reason: Variable name duplicated".to_string(),
        );
        Ok(())
    }

    #[test]
    fn test_match_declared_variables_different_block() -> Result<(), String> {
        let original_code =
            "{const __datadog_test_0 = 666; function z(){const c = a + b();}}{const d = e + f()}"
                .to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string());
        assert!(&rewritten.is_err());
        assert_that!(&rewritten.err()).contains(
            "Cancelling test.js file rewrite. Reason: Variable name duplicated".to_string(),
        );
        Ok(())
    }

    #[test]
    fn test_match_declared_function_param_block() -> Result<(), String> {
        let original_code = "{function z(__datadog_test_0){const c = a + b();}}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string());
        assert!(&rewritten.is_err());
        assert_that!(&rewritten.err()).contains(
            "Cancelling test.js file rewrite. Reason: Variable name duplicated".to_string(),
        );
        Ok(())
    }

    #[test]
    fn test_match_declared_function_param_child_block() -> Result<(), String> {
        let original_code =
            "{const a = b + c(); function z(__datadog_test_0){const d = e + f;}}{const d = e + f()}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string());

        assert!(&rewritten.is_err());
        assert_that!(&rewritten.err()).contains(
            "Cancelling test.js file rewrite. Reason: Variable name duplicated".to_string(),
        );
        Ok(())
    }

    #[test]
    fn test_property_access_inside_binary_operation() -> Result<(), String> {
        let original_code = "{const a = b + c.x}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;

        assert_that(&rewritten.code)
            .contains("let __datadog_test_0, __datadog_test_1;
    const a = (__datadog_test_0 = b, __datadog_test_1 = c.x, _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1))");
        Ok(())
    }

    #[test]
    fn test_insertion_after_use_strict() -> Result<(), String> {
        let original_code = "{'use strict'
        const a = 1 + b}"
            .to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;

        assert_that(&rewritten.code)
            .contains("'use strict';\n    let __datadog_test_0;\n    const a = (__datadog_test_0 = b, _ddiast.plusOperator(1 + __datadog_test_0, 1, __datadog_test_0));");
        Ok(())
    }

    #[test]
    fn test_insertion_after_use_strict_with_semicolon() -> Result<(), String> {
        let original_code = "{\n\n'use strict';
        const a = 1 + b}"
            .to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;

        assert_that(&rewritten.code)
            .contains("'use strict';\n    let __datadog_test_0;\n    const a = (__datadog_test_0 = b, _ddiast.plusOperator(1 + __datadog_test_0, 1, __datadog_test_0));");
        Ok(())
    }
}
