/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use crate::{
    telemetry::TelemetryVerbosity,
    transform::transform_status::{Status, TransformStatus},
    util::{file_name, parse_source_map, FileReader},
    visitor::{block_transform_visitor::BlockTransformVisitor, csi_methods::CsiMethods},
};
use anyhow::{Error, Result};
use std::{
    collections::HashMap,
    io::Read,
    path::{Path, PathBuf},
    str,
    sync::Arc,
};
use swc::{
    common,
    common::{
        comments::Comments,
        errors::{ColorConfig, Handler},
        FileName, FilePathMapping,
    },
    config::{IsModule, SourceMapsConfig},
    ecmascript::ast::*,
    sourcemap::{decode, decode_data_url, DecodedMap, SourceMap, SourceMapBuilder},
    try_with_handler, Compiler, HandlerOpts, SwcComments, TransformOutput,
};

use swc_ecma_parser::{EsConfig, Syntax};
use swc_ecma_visit::VisitMutWith;

const SOURCE_MAP_URL: &str = "# sourceMappingURL=";

pub struct RewrittenOutput {
    pub code: String,
    pub source_map: String,
    pub original_source_map: OriginalSourceMap,
    pub transform_status: Option<TransformStatus>,
}

pub struct OriginalSourceMap {
    pub source: Option<SourceMap>,
    pub source_map_comment: Option<String>,
}

pub struct TransformOutputWithStatus {
    pub output: TransformOutput,
    pub status: TransformStatus,
}

#[derive(Debug)]
pub struct Config {
    pub chain_source_map: bool,
    pub print_comments: bool,
    pub local_var_prefix: String,
    pub csi_methods: CsiMethods,
    pub verbosity: TelemetryVerbosity,
}

pub fn rewrite_js<R: Read>(
    code: String,
    file: &str,
    config: &Config,
    file_reader: &impl FileReader<R>,
) -> Result<RewrittenOutput> {
    let compiler = Compiler::new(Arc::new(common::SourceMap::new(FilePathMapping::empty())));
    try_with_handler(compiler.cm.clone(), default_handler_opts(), |handler| {
        let program = parse_js(&code, file, handler, &compiler)?;

        // extract sourcemap before printing otherwise comments are consumed
        // and looks like it is not possible to read them after compiler.print() invocation
        let original_map = extract_source_map(file, compiler.comments(), file_reader);

        let result = transform_js(program, &code, file, config, &compiler);

        result.map(|transformed| RewrittenOutput {
            code: transformed.output.code,
            source_map: transformed.output.map.unwrap_or_default(),
            original_source_map: original_map,
            transform_status: Some(transformed.status),
        })
    })
}

pub fn print_js(output: &RewrittenOutput, config: &Config) -> String {
    let mut final_source_map: String = String::from(&output.source_map);
    let original_source_map = &output.original_source_map;
    if config.chain_source_map {
        final_source_map = chain_source_maps(&output.source_map, &original_source_map.source)
            .unwrap_or_else(|_| String::from(&output.source_map));
    }

    let final_code = if config.print_comments {
        match &original_source_map.source_map_comment {
            Some(comment) => output.code.replace(comment.as_str(), ""),
            _ => output.code.clone(),
        }
    } else {
        output.code.clone()
    };

    if final_source_map.is_empty() {
        final_code
    } else {
        format!(
            "{}\n//{}data:application/json;base64,{}",
            final_code,
            SOURCE_MAP_URL,
            base64::encode(final_source_map)
        )
    }
}

fn default_handler_opts() -> HandlerOpts {
    HandlerOpts {
        color: ColorConfig::Never,
        skip_filename: false,
    }
}

fn parse_js(
    source: &String,
    file: &str,
    handler: &Handler,
    compiler: &Compiler,
) -> Result<Program> {
    let fm = compiler
        .cm
        .new_source_file(FileName::Real(PathBuf::from(file)), source.into());
    let es_config = EsConfig {
        jsx: false,
        fn_bind: false,
        decorators: false,
        decorators_before_export: false,
        export_default_from: false,
        import_assertions: false,
        private_in_object: false,
        allow_super_outside_method: false,
        allow_return_outside_function: true,
    };

    compiler.parse_js(
        fm,
        handler,
        EsVersion::latest(),
        Syntax::Es(es_config),
        IsModule::Unknown,
        Some(&compiler.comments() as &dyn Comments),
    )
}

fn transform_js(
    mut program: Program,
    code: &str,
    file: &str,
    config: &Config,
    compiler: &Compiler,
) -> Result<TransformOutputWithStatus, Error> {
    let mut transform_status = TransformStatus::not_modified(config);
    let mut block_transform_visitor = BlockTransformVisitor::default(&mut transform_status, config);
    program.visit_mut_with(&mut block_transform_visitor);

    match transform_status.status {
        Status::Modified => compiler
            .print(
                &program,
                file_name(file),
                None,
                false,
                EsVersion::latest(),
                SourceMapsConfig::Bool(true),
                &Default::default(),
                None,
                false,
                config
                    .print_comments
                    .then_some(&compiler.comments().clone() as &dyn Comments),
                true,
                false,
            )
            .map(|output| TransformOutputWithStatus {
                output,
                status: transform_status,
            }),
        Status::NotModified => Ok(TransformOutputWithStatus {
            output: TransformOutput {
                code: code.to_string(),
                map: None,
            },
            status: transform_status,
        }),
        Status::Cancelled => Err(Error::msg(format!(
            "Cancelling {} file rewrite. Reason: {}",
            file,
            transform_status
                .msg
                .unwrap_or_else(|| "unknown".to_string())
        ))),
    }
}

fn chain_source_maps(
    source_map: &String,
    original_map: &Option<SourceMap>,
) -> Result<String, Error> {
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
                            source_idx = Some(sources.get(source).copied().unwrap_or_else(|| {
                                let result = builder.add_source(source);
                                sources.insert(String::from(source), result);
                                result
                            }));
                        }
                        let mut name_idx = None;
                        if original.has_name() {
                            let name = original.get_name().unwrap();
                            name_idx = Some(names.get(name).copied().unwrap_or_else(|| {
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
                let mut source_map_output: Vec<u8> = vec![];
                builder
                    .into_sourcemap()
                    .to_writer(&mut source_map_output)
                    .map(|_| String::from_utf8(source_map_output).unwrap())
                    .map_err(Error::new)
            }
            None => Result::Ok(String::from(source_map)),
        }
    } else {
        Result::Ok(String::from(source_map))
    }
}

fn extract_source_map<R: Read>(
    file_path: &str,
    comments: &SwcComments,
    file_reader: &impl FileReader<R>,
) -> OriginalSourceMap {
    let mut source_map_comment = None;
    let mut source: Option<SourceMap> = None;
    for trailing in comments.trailing.iter() {
        for comment in trailing.iter() {
            let trim_comment = comment.text.trim();
            if trim_comment.starts_with(SOURCE_MAP_URL) {
                source_map_comment = Some(comment.text.clone());
                let url = trim_comment.get(SOURCE_MAP_URL.len()..).unwrap();
                source = decode_data_url(url)
                    .map_err(Error::new)
                    .or_else(|_| {
                        let source_path = PathBuf::from(url);
                        let final_path = if source_path.is_absolute() {
                            source_path
                        } else {
                            let folder = file_reader.parent(Path::new(file_path)).unwrap();
                            folder.join(source_path)
                        };

                        decode(file_reader.read(&final_path)?)
                    })
                    .ok()
                    .and_then(|it| match it {
                        DecodedMap::Regular(source) => Some(source),
                        _ => None,
                    });
            }
        }
    }

    OriginalSourceMap {
        source,
        source_map_comment,
    }
}

#[cfg(test)]
pub fn debug_js(code: String) -> Result<RewrittenOutput> {
    use crate::util::DefaultFileReader;

    let compiler = Compiler::new(Arc::new(common::SourceMap::new(FilePathMapping::empty())));
    return try_with_handler(compiler.cm.clone(), default_handler_opts(), |handler| {
        let js_file = "debug.js".to_string();
        let program = parse_js(&code, &js_file, handler, &compiler)?;

        print!("{:#?}", program);

        let source_map_reader = DefaultFileReader {};
        let original_map =
            extract_source_map(js_file.as_str(), &compiler.comments(), &source_map_reader);

        let print_result = compiler.print(
            &program,
            file_name(&js_file),
            None,
            false,
            EsVersion::latest(),
            SourceMapsConfig::Bool(true),
            &Default::default(),
            None,
            false,
            Some(compiler.comments() as &dyn Comments),
            true,
            false,
        );

        print_result.map(|printed| RewrittenOutput {
            code: printed.code,
            source_map: printed.map.unwrap(),
            original_source_map: original_map,
            transform_status: None,
        })
    });
}
