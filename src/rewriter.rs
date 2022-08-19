use crate::{
    rewriter::AssignOp::{AddAssign, Assign},
    util::{create_file, file_name, parse_source_map},
};
use anyhow::{Error, Result};
use std::{
    borrow::Borrow,
    collections::HashMap,
    fs::File,
    io::Write,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};
use swc::{
    atoms::JsWord,
    common,
    common::{
        comments::Comments,
        errors::{ColorConfig, Handler},
        util::take::Take,
        FileName, FilePathMapping, Span,
    },
    config::{IsModule, SourceMapsConfig},
    ecmascript::ast::*,
    sourcemap::{decode, decode_data_url, DecodedMap, SourceMap, SourceMapBuilder},
    try_with_handler, Compiler, HandlerOpts, SwcComments, TransformOutput,
};
use crate::util::{create_file, file_name, parse_source_map};
use crate::block_transform_visitor::BlockTransformVisitor;
//use crate::transform_visitor::TransformVisitor;
use anyhow::{Error, Result};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use swc::common::comments::Comments;
use swc::common::errors::{ColorConfig, Handler};
use swc::common::{FileName, FilePathMapping};
use swc::config::{IsModule, SourceMapsConfig};
use swc::ecmascript::ast::*;
use swc::sourcemap::{decode, decode_data_url, DecodedMap, SourceMap, SourceMapBuilder};
use swc::{common, try_with_handler, Compiler, HandlerOpts, SwcComments, TransformOutput};
use swc_ecma_parser::{EsConfig, Syntax};
use swc_ecma_visit::VisitMutWith;


const SOURCE_MAP_URL: &str = "# sourceMappingURL=";


pub struct RewrittenOutput {
    pub code: String,
    pub source_map: String,
    pub original_map: Option<SourceMap>,
}

pub fn debug_js(code: String) -> Result<(), Error> {
    let compiler = Compiler::new(Arc::new(common::SourceMap::new(FilePathMapping::empty())));
    return try_with_handler(compiler.cm.clone(), default_handler_opts(), |handler| {
        let js_file = "debug.js".to_string();
        let program = parse_js(code, &js_file, handler, compiler.borrow())?;
        print!("{:#?}", program);
        Ok(())
    });
}

pub fn rewrite_js(code: String, file: String) -> Result<RewrittenOutput> {
    let compiler = Compiler::new(Arc::new(common::SourceMap::new(FilePathMapping::empty())));
    return try_with_handler(compiler.cm.clone(), default_handler_opts(), |handler| {
        let program = parse_js(code, file.as_str(), handler, compiler.borrow())?;
        let transformed = transform_js(program, file.as_str(), compiler.borrow())?;
        Ok(RewrittenOutput {
            code: transformed.code,
            source_map: transformed.map.unwrap(),
            original_map: extract_source_map(
                Path::new(file.as_str()).parent().unwrap(),
                compiler.comments(),
            ),
        })
    });
}

pub fn print_js(output: RewrittenOutput, source_map: Option<String>) -> String {
    match source_map {
        Some(target_path) => create_file(Path::new(target_path.as_str()))
            .and_then(|target_file| {
                chain_source_map(target_file, output.source_map, output.original_map)
            })
            .map(|()| format!("{}\n//# sourceMappingURL={}", output.code, target_path))
            .unwrap_or(output.code),
        None => output.code,
    }
}

fn default_handler_opts() -> HandlerOpts {
    HandlerOpts {
        color: ColorConfig::Never,
        skip_filename: false,
    }
}

fn parse_js(source: String, file: &str, handler: &Handler, compiler: &Compiler) -> Result<Program> {
    let fm = compiler
        .cm
        .new_source_file(FileName::Real(PathBuf::from(file)), source.as_str().into());
    compiler.parse_js(
        fm,
        handler,
        EsVersion::Es2020,
        Syntax::Es(EsConfig::default()),
        IsModule::Unknown,
        Some(compiler.comments() as &dyn Comments),
    )
}

fn transform_js(mut program: Program, file: &str, compiler: &Compiler) -> Result<TransformOutput> {
    //program.visit_mut_with(&mut TransformVisitor {counter: 0});
    program.visit_mut_with(&mut BlockTransformVisitor {});
    compiler.print(
        &program,
        file_name(file),
        None,
        false,
        EsVersion::Es2020,
        SourceMapsConfig::Bool(true),
        &Default::default(),
        None,
        false,
        None,
    )
}

fn chain_source_map(
    mut target_file: File,
    source_map: String,
    original_map: Option<SourceMap>,
) -> Result<()> {
    if let Some(new_source) = parse_source_map(Some(source_map.as_str())) {
        match original_map {
            Some(original_source) => {
                let mut builder = SourceMapBuilder::new(None);
                let mut sources: HashMap<String, u32> = HashMap::new();
                let mut names: HashMap<String, u32> = HashMap::new();
                for token in new_source.tokens() {
                    let original_token =
                        original_source.lookup_token(token.get_src_line(), token.get_src_col());
                    if let Some(original) = original_token {
                        let mut source_idx = None;
                        if original.has_source() {
                            let source = original.get_source().unwrap();
                            source_idx =
                                Some(sources.get(source).map(|it| *it).unwrap_or_else(|| {
                                    let result = builder.add_source(source);
                                    sources.insert(String::from(source), result);
                                    result
                                }));
                        }
                        let mut name_idx = None;
                        if original.has_name() {
                            let name = original.get_name().unwrap();
                            name_idx = Some(names.get(name).map(|it| *it).unwrap_or_else(|| {
                                let result = builder.add_name(name);
                                names.insert(String::from(name), result);
                                result
                            }));
                        }
                        builder.add_raw(
                            token.get_dst_line(),
                            token.get_dst_col(),
                            original.get_src_line(),
                            original.get_src_col(),
                            source_idx,
                            name_idx,
                        );
                    }
                }
                builder
                    .into_sourcemap()
                    .to_writer(target_file)
                    .map_err(Error::new)
            }
            None => target_file
                .write_all(source_map.as_bytes())
                .map_err(Error::new),
        }
    } else {
        Result::Ok(())
    }
}

fn extract_source_map(folder: &Path, comments: &SwcComments) -> Option<SourceMap> {
    for trailing in comments.trailing.iter() {
        for comment in trailing.iter() {
            if comment.text.starts_with(SOURCE_MAP_URL) {
                let url = comment.text.get(SOURCE_MAP_URL.len()..).unwrap();
                return decode_data_url(url)
                    .map_err(Error::new)
                    .or_else(|_| {
                        let source_path = PathBuf::from(url);
                        let final_path = if source_path.is_absolute() {
                            source_path
                        } else {
                            folder.join(source_path)
                        };
                        let file = File::open(final_path)?;
                        return decode(file);
                    })
                    .ok()
                    .and_then(|it| match it {
                        DecodedMap::Regular(source) => Some(source),
                        _ => None,
                    });
            }
        }
    }
    return None;
}