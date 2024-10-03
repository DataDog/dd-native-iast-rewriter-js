/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/

#[cfg(test)]
mod tests {

    use speculoos::{assert_that, prelude::ContainingIntoIterAssertions, string::StrAssertions};

    use anyhow::Error;

    use crate::{rewriter::debug_js, tests::rewrite_js};

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
            .contains("const result = (__datadog_test_0 = _ddiast.plusOperator(a + 'b', a, 'b'), _ddiast.plusOperator(__datadog_test_0 + 'c', __datadog_test_0, 'c'));");
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
        assert_that(&rewritten.code).contains("const result = _ddiast.plusOperator(a + b, a, b);");
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
        assert_that(&rewritten.code).contains("let __datadog_test_0, __datadog_test_1;
    const result = (__datadog_test_1 = (__datadog_test_0 = _ddiast.plusOperator(a + b, a, b), _ddiast.plusOperator(__datadog_test_0 + c, __datadog_test_0, c)), _ddiast.plusOperator(__datadog_test_1 + d, __datadog_test_1, d));");
        Ok(())
    }

    #[test]
    fn test_multiple_plus_and_func() -> Result<(), String> {
        let original_code = "{const result = a + b + c() + d}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("let __datadog_test_0, __datadog_test_1, __datadog_test_2;
    const result = (__datadog_test_2 = (__datadog_test_0 = _ddiast.plusOperator(a + b, a, b), __datadog_test_1 = c(), _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1)), _ddiast.plusOperator(__datadog_test_2 + d, __datadog_test_2, d));");
        Ok(())
    }

    #[test]
    fn test_multiple_plus_and_inlined_func() -> Result<(), String> {
        let original_code =
            "{const result = a + b + (function(a){return a + 'hello';})(a)}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("const result = (__datadog_test_0 = _ddiast.plusOperator(a + b, a, b), __datadog_test_1 = (function(a) {
        return _ddiast.plusOperator(a + 'hello', a, 'hello');
    })(a), _ddiast.plusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1));");
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
            .contains("return (__datadog_test_0 = _ddiast.plusOperator(a + '.', a, '.'), _ddiast.plusOperator(__datadog_test_0 + a, __datadog_test_0, a));");

        assert_that(&rewritten.code)
            .contains("return (__datadog_test_0 = _ddiast.plusOperator(a + '-', a, '-'), _ddiast.plusOperator(__datadog_test_0 + a, __datadog_test_0, a));");

        assert_that!(&rewritten.code).contains("let __datadog_test_0;");

        assert_that(&rewritten.code)
            .contains("const c = a === 'hello' ? (__datadog_test_0 = fn1(b), _ddiast.plusOperator('foo' + __datadog_test_0, 'foo', __datadog_test_0)) : (__datadog_test_0 = fn2(b), _ddiast.plusOperator('bar' + __datadog_test_0, 'bar', __datadog_test_0));
    return (__datadog_test_0 = fn2(a), _ddiast.plusOperator(__datadog_test_0 + c, __datadog_test_0, c));");
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
            .contains("let __datadog_test_0;
    let b;
    const a = (__datadog_test_0 = (b = '_b_', _ddiast.plusOperator(b + c, b, c)), _ddiast.plusOperator('a' + __datadog_test_0, 'a', __datadog_test_0));");
        Ok(())
    }

    #[test]
    fn test_plus_inside_array() -> Result<(), String> {
        let original_code = "{const a = [a + b, c + d];}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains(
            "const a = [
        _ddiast.plusOperator(a + b, a, b),
        _ddiast.plusOperator(c + d, c, d)
    ];",
        );
        Ok(())
    }

    #[test]
    fn test_plus_inside_object_assign() -> Result<(), String> {
        let original_code = "{const a = Object.assign({[prop]: a + b})}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains(
            "const a = Object.assign({\n        [prop]: _ddiast.plusOperator(a + b, a, b)\n    });",
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
            .contains("'use strict';\n    const a = _ddiast.plusOperator(1 + b, 1, b);");
        Ok(())
    }

    #[test]
    fn test_insertion_after_use_strict_with_semicolon() -> Result<(), String> {
        let original_code = "{\n\n/*This is a comment*/'use strict';
        const a = 1 + b}"
            .to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("'use strict';\n    const a = _ddiast.plusOperator(1 + b, 1, b);");
        Ok(())
    }

    // Is this a swc bug?
    #[test]
    #[ignore]
    fn test_es6_destructuring_swc_bug() -> Result<(), String> {
        let original_code = "let [a, b, ,] = f();".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;

        assert_that(&rewritten.code).contains("let [a, b, ,] = f();"); // result after rewrite: let [a, b, ] = f();
        Ok(())
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
    fn test_expression_not_modified_at_the_end() -> Result<(), String> {
        let original_code = "{const a = b + c\nconst d = 'a' + 'b'\n}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains(
            "const a = _ddiast.plusOperator(b + c, b, c);
    const d = 'a' + 'b';",
        );
        Ok(())
    }
}
