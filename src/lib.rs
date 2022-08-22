#![deny(clippy::all)]

mod block_transform_visitor;
mod operation_transform_visitor;
mod rewriter;
mod transform_visitor;
mod util;
mod visitor_util;

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
    use spectral::{assert_that, string::StrAssertions};

    use anyhow::Error;
    use spectral::assert_that;
    use spectral::string::StrAssertions;
    use std::env;

    use crate::rewrite_js;
    use crate::rewriter::debug_js;

    #[cfg(test)]
    #[ctor::ctor]
    fn init() {
        match env::var("DD_LOCAL_VAR_NAME_HASH") {
            Err(_) => {
                env::set_var("DD_LOCAL_VAR_NAME_HASH", "test");
            }
            Ok(_) => {}
        }
    }

    #[test]
    fn test_paren_stmt() -> Result<(), Error> {
        let original_code = "(a,b,c)".to_string();
        return debug_js(original_code);
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
        assert_that(&rewritten.code).contains("(global._ddiast.twoItemsPlusOperator(this.height + this.width, this.height, this.width))");
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
        assert_that(&rewritten.code).contains("(__datadog_test_0 = this.w(), global._ddiast.twoItemsPlusOperator(this.height + __datadog_test_0, this.height, __datadog_test_0))");
        Ok(())
    }

    #[test]
    fn test_simple_plus() -> Result<(), String> {
        let original_code = "{const result = a + b}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("(global._ddiast.twoItemsPlusOperator(a + b, a, b))");
        Ok(())
    }

    #[test]
    fn test_simple_plus_call() -> Result<(), String> {
        let original_code = "{const result = a + b()}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("(__datadog_test_0 = b(), global._ddiast.twoItemsPlusOperator(a + __datadog_test_0, a, __datadog_test_0))");
        Ok(())
    }

    #[test]
    fn test_multiple_plus() -> Result<(), String> {
        let original_code = "{const result = a + b + c + d}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code)
            .contains("(global._ddiast.fourItemsPlusOperator(a + b + c + d, a, b, c, d))");
        Ok(())
    }

    #[test]
    fn test_multiple_plus_and_func() -> Result<(), String> {
        let original_code = "{const result = a + b + c() + d}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("(__datadog_test_0 = c(), global._ddiast.fourItemsPlusOperator(a + b + __datadog_test_0 + d, a, b, __datadog_test_0, d))");
        Ok(())
    }

    #[test]
    fn test_multiple_plus_and_inlined_func() -> Result<(), String> {
        let original_code =
            "{const result = a + b + (function(a){return a + 'epa';})(a)}".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains(
            "global._ddiast.threeItemsPlusOperator(a + b + __datadog_test_0, a, b, __datadog_test_0)",
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
        assert_that(&rewritten.code).contains(
            "global._ddiast.twoItemsPlusOperator(__datadog_test_0 + c, __datadog_test_0, c)",
        );
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
            .contains("return (__datadog_test_0 = fn2(a), __datadog_test_1 = fn3(c), global._ddiast.twoItemsPlusOperator(__datadog_test_0 + __datadog_test_1, __datadog_test_0, __datadog_test_1))");
        Ok(())
    }
}
