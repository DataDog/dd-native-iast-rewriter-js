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
    use spectral::assert_that;
    use spectral::option::OptionAssertions;
    use spectral::string::StrAssertions;
    use std::fs;
    use swc::sourcemap::SourceMap;
    use tempfile::NamedTempFile;

    use crate::{print_js, rewrite_js};

    #[test]
    fn test_source_maps() -> Result<(), String> {
        let folder = std::env::current_dir()
            .map(|cwd| cwd.join("samples"))
            .map_err(|e| e.to_string())?;

        let js_file = folder.join("login.js");
        let original_code = fs::read_to_string(js_file.clone()).map_err(|e| e.to_string())?;
        let rewritten = rewrite_js(original_code, String::from(js_file.to_str().unwrap()))
            .map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("global._ddiast.templateLiteralOperator(`SELECT");
        assert_that(&rewritten.code.find("# sourceMappingURL=")).is_none();

        let source_map_file = NamedTempFile::new().map_err(|e| e.to_string())?;
        let result = print_js(
            rewritten,
            Some(String::from(
                source_map_file
                    .path()
                    .to_str()
                    .ok_or("Fail to get source path")?,
            )),
        );
        assert_that(&result).contains("# sourceMappingURL=");
        let source_map = SourceMap::from_reader(source_map_file).map_err(|e| e.to_string())?;
        let token = source_map.lookup_token(35, 25).unwrap();
        assert_that(&token.get_src_line()).is_equal_to(31);
        assert_that(&token.get_source().unwrap()).contains("login.ts");

        Ok(())
    }

    #[test]
    fn test_multiple_plus() -> Result<(), String> {
        let original_code = "const result = a + b + c + d".to_string();
        let js_file = "test.js".to_string();
        let rewritten = rewrite_js(original_code, js_file).map_err(|e| e.to_string())?;
        assert_that(&rewritten.code).contains("global._ddiast.fourItemsPlusOperator(");
        Ok(())
    }
}
