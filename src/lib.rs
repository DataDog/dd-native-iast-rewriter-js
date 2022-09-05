#![deny(clippy::all)]
/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
mod rewriter;
mod util;
mod visitor;

#[macro_use]
extern crate napi_derive;

use crate::rewriter::{print_js, rewrite_js};
use napi::{Error, Status};

#[napi]
pub struct Rewriter {}

#[napi]
impl Rewriter {
    #[napi(constructor)]
    pub fn new() -> Self {
        Rewriter {}
    }

    #[napi]
    pub fn rewrite(
        &self,
        code: String,
        file: String,
        source_map: Option<String>,
    ) -> napi::Result<String> {
        return rewrite_js(code, file)
            .map(|result| print_js(result, source_map))
            .map_err(|e| Error::new(Status::Unknown, format!("{}", e)));
    }
}

#[cfg(test)]
mod tests {
    use spectral::{assert_that, prelude::ContainingIntoIterAssertions, string::StrAssertions};

    use anyhow::Error;
    use std::env;

    use crate::{
        rewrite_js, rewriter::debug_js, visitor::visitor_util::DD_LOCAL_VAR_NAME_HASH_ENV_NAME,
    };

    #[cfg(test)]
    #[ctor::ctor]
    fn init() {
        match env::var(DD_LOCAL_VAR_NAME_HASH_ENV_NAME) {
            Err(_) => {
                env::set_var(DD_LOCAL_VAR_NAME_HASH_ENV_NAME, "test");
            }
            Ok(_) => {}
        }
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
            "global._ddiast.plusOperator(this.height + this.width, this.height, this.width)",
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
        assert_that(&rewritten.code).contains("(__datadog_test_0 = this.w(), global._ddiast.plusOperator(this.height + __datadog_test_0, this.height, __datadog_test_0))");
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
    #[ignore]
    fn test_variable_plus_literals() -> Result<(), String> {
        let original_code = "{const result = a + 'b' + 'c'}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("const result = global._ddiast.plusOperator(a + 'b' + 'c', a, 'b' + 'c')");
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
        assert_that(&rewritten.code).contains("global._ddiast.plusOperator(a + b, a, b)");
        Ok(())
    }

    #[test]
    fn test_simple_plus_call() -> Result<(), String> {
        let original_code = "{const result = a + b()}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("(__datadog_test_0 = b(), global._ddiast.plusOperator(a + __datadog_test_0, a, __datadog_test_0))");
        Ok(())
    }

    #[test]
    fn test_multiple_plus() -> Result<(), String> {
        let original_code = "{const result = a + b + c + d}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("global._ddiast.plusOperator(a + b + c + d, a, b, c, d)");
        Ok(())
    }

    #[test]
    fn test_multiple_plus_and_func() -> Result<(), String> {
        let original_code = "{const result = a + b + c() + d}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("(__datadog_test_0 = c(), global._ddiast.plusOperator(a + b + __datadog_test_0 + d, a, b, __datadog_test_0, d))");
        Ok(())
    }

    #[test]
    fn test_multiple_plus_and_inlined_func() -> Result<(), String> {
        let original_code =
            "{const result = a + b + (function(a){return a + 'epa';})(a)}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains(
            "global._ddiast.plusOperator(a + b + __datadog_test_0, a, b, __datadog_test_0)",
        );
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
            .contains("global._ddiast.plusOperator(__datadog_test_2 + c, __datadog_test_2, c)");
        Ok(())
    }

    #[test]
    fn test_multiple_plus_inside_if_and_func2() -> Result<(), String> {
        let original_code = "
        function fn1(a){
            return a + '.' + a;
        }
        function fn2(a){
            return a + '-' + a;
        }
        function fn3(a){
            return a;
        }
        
        const fn = function(a, b){
            const c = a === 'hello' ? 'foo' + fn1(b) : 'bar' + fn2(b);
            return fn2(a) + fn3(c);
        }"
        .to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("return (__datadog_test_2 = fn2(a), __datadog_test_3 = fn3(c), global._ddiast.plusOperator(__datadog_test_2 + __datadog_test_3, __datadog_test_2, __datadog_test_3))");
        Ok(())
    }

    #[test]
    fn test_simple_plus_with_multiply() -> Result<(), String> {
        let original_code = "{const result = a + b * c}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("global._ddiast.plusOperator(a + b * c, a, b * c)");
        Ok(())
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
    fn test_plus_and_block() -> Result<(), String> {
        let original_code = "{let b;const a = 'a' + (b = '_b_', b + c);}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("let __datadog_test_0;\n    let b;\n    const a = (__datadog_test_0 = (b = '_b_', global._ddiast.plusOperator(b + c, b, c)), global._ddiast.plusOperator('a' + __datadog_test_0, 'a', __datadog_test_0));");
        Ok(())
    }

    #[test]
    fn test_plus_inside_array() -> Result<(), String> {
        let original_code = "{const a = [a + b, c + d];}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("const a = [\n        global._ddiast.plusOperator(a + b, a, b),\n        global._ddiast.plusOperator(c + d, c, d)\n    ]");
        Ok(())
    }

    #[test]
    fn test_plus_inside_object_assign() -> Result<(), String> {
        let original_code = "{const a = Object.assign({[prop]: a + b})}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("const a = Object.assign({\n        [prop]: global._ddiast.plusOperator(a + b, a, b)\n    })");
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
}
