#![deny(clippy::all)]

mod rewriter;
mod util;

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

    use crate::rewrite_js;

    #[test]
    fn test_multiple_plus() -> Result<(), String> {
        let original_code = "const result = a + b + c + d".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("global._ddiast.fourItemsPlusOperator(");
        Ok(())
    }
}
