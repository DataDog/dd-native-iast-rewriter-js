/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/

#[cfg(test)]
mod tests {
    use spectral::{assert_that, string::StrAssertions};
    use std::fs;

    use swc::sourcemap::{decode_data_url, DecodedMap};

    use crate::{
        rewriter::{print_js, rewrite_js, RewrittenOutput},
        tests::{get_default_config, get_test_resources_folder},
    };

    #[derive(Clone)]
    struct TokenChecking {
        dst_line: u32,
        dst_col: u32,
        src_line: u32,
        src_col: u32,
    }

    const UNCHAINED_TOKENS: [TokenChecking; 3] = [
        TokenChecking {
            dst_line: 9,
            dst_col: 75,
            src_line: 8,
            src_col: 29,
        },
        TokenChecking {
            dst_line: 12,
            dst_col: 17,
            src_line: 11,
            src_col: 13,
        },
        TokenChecking {
            dst_line: 15,
            dst_col: 36,
            src_line: 14,
            src_col: 11,
        },
    ];

    const CHAINED_TOKENS: [TokenChecking; 3] = [
        TokenChecking {
            dst_line: 9,
            dst_col: 75,
            src_line: 5,
            src_col: 15,
        },
        TokenChecking {
            dst_line: 12,
            dst_col: 17,
            src_line: 9,
            src_col: 9,
        },
        TokenChecking {
            dst_line: 15,
            dst_col: 36,
            src_line: 13,
            src_col: 9,
        },
    ];

    const SOURCE_MAP_URL: &str = "# sourceMappingURL=";
    const SOURCE_MAP_URL_COMMENT: &str = "//# sourceMappingURL=";

    fn get_sourcemap_from_printed_js(printed_js: String) -> Option<DecodedMap> {
        for line in printed_js.split("\n") {
            let trim_line = line.trim();
            if trim_line.starts_with(SOURCE_MAP_URL_COMMENT) {
                let url = trim_line.get(SOURCE_MAP_URL_COMMENT.len()..).unwrap();
                return Some(decode_data_url(url).unwrap());
            }
        }
        return None;
    }

    fn get_rewritten_js(file_js: &str) -> Result<RewrittenOutput, String> {
        let sourcemap_resources_folder = get_test_resources_folder()
            .map(|resources_folder| resources_folder.join("sourcemap"))
            .map_err(|e| e.to_string())?;
        let js_file_name = String::from(file_js);
        let js_file_to_rewrite = sourcemap_resources_folder.join(js_file_name);
        let original_code =
            fs::read_to_string(js_file_to_rewrite.clone()).map_err(|e| e.to_string())?;
        rewrite_js(
            original_code,
            String::from(js_file_to_rewrite.to_str().unwrap()),
            get_default_config(true),
        )
        .map_err(|e| e.to_string())
    }

    fn check_sourcemap_tokens(js_code: String, tokens_to_check: Vec<TokenChecking>) {
        match get_sourcemap_from_printed_js(js_code) {
            Some(sourcemap) => {
                for check_token in tokens_to_check {
                    let token = sourcemap
                        .lookup_token(check_token.dst_line, check_token.dst_col)
                        .unwrap();
                    assert_that(&token.get_src_line()).is_equal_to(check_token.src_line);
                    assert_that(&token.get_src_col()).is_equal_to(check_token.src_col);
                }
            }
            None => panic!("No sourcemap"),
        }
    }

    #[test]
    fn test_source_maps_unchained_embedded() -> Result<(), String> {
        let rewritten = get_rewritten_js("StrUtil_embedded.js")?;
        let result = print_js(rewritten, false);
        assert_that(&result).contains(SOURCE_MAP_URL);
        check_sourcemap_tokens(result, UNCHAINED_TOKENS.to_vec());
        Ok(())
    }

    #[test]
    fn test_source_maps_unchained_external() -> Result<(), String> {
        let rewritten = get_rewritten_js("StrUtil_external.js")?;
        let result = print_js(rewritten, false);
        assert_that(&result).contains(SOURCE_MAP_URL);
        check_sourcemap_tokens(result, UNCHAINED_TOKENS.to_vec());
        Ok(())
    }

    #[test]
    fn test_source_maps_unchained_without_original_source_map() -> Result<(), String> {
        let rewritten = get_rewritten_js("StrUtil_without_sm.js")?;
        let result = print_js(rewritten, false);
        assert_that(&result).contains(SOURCE_MAP_URL);
        check_sourcemap_tokens(result, UNCHAINED_TOKENS.to_vec());
        Ok(())
    }

    #[test]
    fn test_source_maps_chained_embedded() -> Result<(), String> {
        let rewritten = get_rewritten_js("StrUtil_embedded.js")?;
        let result = print_js(rewritten, true);
        assert_that(&result).contains(SOURCE_MAP_URL);
        check_sourcemap_tokens(result, CHAINED_TOKENS.to_vec());
        Ok(())
    }

    #[test]
    fn test_source_maps_chained_external() -> Result<(), String> {
        let rewritten = get_rewritten_js("StrUtil_external.js")?;
        let result = print_js(rewritten, true);
        assert_that(&result).contains(SOURCE_MAP_URL);
        check_sourcemap_tokens(result, CHAINED_TOKENS.to_vec());
        Ok(())
    }

    #[test]
    fn test_source_maps_chained_without_original_source_map() -> Result<(), String> {
        let rewritten = get_rewritten_js("StrUtil_without_sm.js")?;
        let result = print_js(rewritten, true);
        assert_that(&result).contains(SOURCE_MAP_URL);
        check_sourcemap_tokens(result, UNCHAINED_TOKENS.to_vec());
        Ok(())
    }
}
