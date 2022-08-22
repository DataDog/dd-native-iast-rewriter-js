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
use swc_ecma_parser::{EsConfig, Syntax};
use swc_ecma_visit::{Visit, VisitMut, VisitMutWith};

const SOURCE_MAP_URL: &str = "# sourceMappingURL=";
const NODE_GLOBAL: &str = "global";
const DD_GLOBAL_NAMESPACE: &str = "_ddiast";
const DD_TWO_ITEMS_PLUS_OPERATOR: &str = "twoItemsPlusOperator";
const DD_THREE_ITEMS_PLUS_OPERATOR: &str = "threeItemsPlusOperator";
const DD_FOUR_ITEMS_PLUS_OPERATOR: &str = "fourItemsPlusOperator";
const DD_FIVE_ITEMS_PLUS_OPERATOR: &str = "fiveItemsPlusOperator";
const DD_ANY_PLUS_OPERATOR: &str = "anyPlusOperator";

pub struct RewrittenOutput {
    pub code: String,
    pub source_map: String,
    pub original_map: Option<SourceMap>,
}

pub struct TransformVisitor {}

impl TransformVisitor {}

impl Visit for TransformVisitor {}

impl VisitMut for TransformVisitor {
    fn visit_mut_assign_expr(&mut self, assign: &mut AssignExpr) {
        assign.visit_mut_children_with(self);
        if assign.op == AddAssign {
            assign.map_with_mut(|assign| to_dd_assign_expr(assign));
        }
    }

    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        expr.visit_mut_children_with(self);
        match expr {
            Expr::Bin(binary) => {
                if binary.op == BinaryOp::Add {
                    expr.map_with_mut(|bin| to_dd_binary_expr(bin));
                }
            }
            Expr::Tpl(tpl) => {
                if !tpl.exprs.is_empty() {
                    expr.map_with_mut(|tpl| to_dd_tpl_expr(tpl));
                }
            }
            _ => {}
        };
    }
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
    program.visit_mut_with(&mut TransformVisitor {});
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

fn to_dd_assign_expr(assign: AssignExpr) -> AssignExpr {
    let span = assign.span;
    let op = assign.op;
    let left = assign.left;
    let right = assign.right;

    return match left {
        PatOrExpr::Pat(_) => AssignExpr {
            span,
            op,
            left,
            right,
        },
        PatOrExpr::Expr(left_expr) => {
            let left = *left_expr;
            let callee = dd_global_method_invocation(span, two_items_plus_operator);
            let args = vec![
                ExprOrSpread {
                    spread: None,
                    expr: Box::new(left.clone()),
                },
                ExprOrSpread {
                    spread: None,
                    expr: right,
                },
            ];
            let binary = Expr::Call(CallExpr {
                span,
                callee,
                args,
                type_args: None,
            });
            return AssignExpr {
                span,
                op: Assign,
                left: PatOrExpr::Expr(Box::new(left)),
                right: Box::new(binary),
            };
        }
    };
}

fn is_call_one_of_the_dd_methods_provided(call: &CallExpr, dd_methods: Vec<&str>) -> bool {
    let global_string = JsWord::from(NODE_GLOBAL);
    let dd_global_string = JsWord::from(DD_GLOBAL_NAMESPACE);
    let mut is_dd_method = false;
    let mut is_dd_global = false;
    let mut is_node_global = false;
    let mut coded_methods = Vec::new();
    let dd_methods_iter = dd_methods.iter();
    let callee: &MemberExpr;

    match &call.callee {
        Callee::Expr(call_calle) => {
            if let Expr::Member(c) = &**call_calle {
                callee = c;
            } else {
                return false;
            }
        }
        _ => {
            return false;
        }
    }

    for method in dd_methods_iter {
        coded_methods.push(JsWord::from(*method));
    }

    let x = callee.deref();
    if let MemberProp::Ident(ident) = &x.prop {
        if coded_methods.contains(&ident.sym) {
            is_dd_method = true;
        }
    }

    if let Expr::Member(member) = &x.obj.deref() {
        if let MemberProp::Ident(ident) = &member.prop {
            is_dd_global = ident.sym == dd_global_string;
        }

        if let Expr::Ident(ident) = &member.obj.deref() {
            is_node_global = ident.sym == global_string;
        }
    }

    return is_dd_method && is_dd_global && is_node_global;
}

fn is_dd_method(call: &CallExpr) -> bool {
    is_call_one_of_the_dd_methods_provided(
        call,
        vec![
            DD_TWO_ITEMS_PLUS_OPERATOR,
            DD_THREE_ITEMS_PLUS_OPERATOR,
            DD_FOUR_ITEMS_PLUS_OPERATOR,
            DD_FIVE_ITEMS_PLUS_OPERATOR,
            DD_ANY_PLUS_OPERATOR,
        ],
    )
}

fn to_dd_binary_expr(expr: Expr) -> Expr {
    match expr {
        Expr::Bin(binary) => {
            let span = binary.span;
            let op = binary.op;
            let left = binary.left;
            let right = binary.right;
            let mut right_node_pushed: bool = false;

            if let Expr::Lit(_) = right.deref() {
                match left.deref() {
                    Expr::Lit(_) | Expr::Bin(_) => {
                        return Expr::Bin(BinExpr {
                            span,
                            op,
                            left,
                            right,
                        });
                    }
                    _ => {}
                }
            }

            let mut args: Vec<ExprOrSpread> = Vec::new();

            //Previous iteration was one of our methods. Clone args since we are going to modify them
            if let Expr::Call(call) = left.deref() {
                if is_dd_method(call) {
                    args = call.args.clone();
                }
            } else if let Expr::Call(call) = right.deref() {
                if is_dd_method(call) {
                    args = call.args.clone();
                    right_node_pushed = true;
                    args.insert(
                        0,
                        ExprOrSpread {
                            spread: None,
                            expr: left.clone(),
                        },
                    );
                }
            }

            //When this point reaches without args is because there was not a call to our methods
            if args.is_empty() {
                args.push(ExprOrSpread {
                    spread: None,
                    expr: left,
                });
            } else {
                //Handling parameters for our methods
                let last = args.last().unwrap().clone();

                if let Expr::Lit(_) = right.deref() {
                    match &*last.expr {
                        Expr::Lit(_) => {
                            //Last parameter passed to our method was literal and new one is literal. Then create binary with both
                            //Remove previous
                            args.pop();
                            //Add the new one
                            args.push(ExprOrSpread {
                                spread: None,
                                expr: Box::new(Expr::Bin(BinExpr {
                                    span,
                                    op,
                                    left: last.expr.clone(),
                                    right: right.clone(),
                                })),
                            });

                            right_node_pushed = true;
                        }
                        Expr::Bin(last_bin) => {
                            //Previous parameter was a binary of literals
                            if let Expr::Lit(_) = last_bin.left.deref() {
                                args.pop();

                                args.push(ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Bin(BinExpr {
                                        span,
                                        op,
                                        left: last_bin.left.clone(),
                                        right: Box::from(Expr::Bin(BinExpr {
                                            span,
                                            op,
                                            left: Box::new(*last_bin.right.clone()),
                                            right: right.clone(),
                                        })),
                                    })),
                                });

                                right_node_pushed = true;
                            }
                        }
                        _ => {}
                    }
                }
            }

            if !right_node_pushed {
                args.push(ExprOrSpread {
                    spread: None,
                    expr: right,
                });
            }

            Expr::Call(CallExpr {
                span,
                callee: get_plus_operator_based_on_num_of_args_for_span(args.len(), span),
                args,
                type_args: None,
            })
        }
        other => other,
    }
}

fn to_dd_tpl_expr(expr: Expr) -> Expr {
    let original_expr = expr.clone();
    match expr {
        Expr::Tpl(tpl) => {
            let span = tpl.span;
            let callee = dd_global_method_invocation(span, template_literal_operator);
            let mut args: Vec<ExprOrSpread> = Vec::new();
            let mut index = 0;
            let mut exprs = tpl.exprs.clone();
            let mut all_literals: bool = true;
            for quasi in tpl.quasis {
                let mut expr_args = Vec::new();
                expr_args.push(TplElement {
                    span: quasi.span,
                    tail: true,
                    cooked: quasi.cooked.clone(),
                    raw: quasi.raw,
                });
                let expr = Tpl {
                    span: tpl.span,
                    quasis: expr_args,
                    exprs: Vec::new(),
                };
                if quasi.cooked.clone().unwrap() != JsWord::from("") {
                    args.push(ExprOrSpread {
                        spread: None,
                        expr: Box::new(Expr::Tpl(expr)),
                    });
                }
                if !quasi.tail {
                    match *exprs[index] {
                        Expr::Lit(_) => {
                            //Nothing to do here
                        }
                        _ => {
                            all_literals = false;
                        }
                    }
                    args.push(ExprOrSpread {
                        spread: None,
                        expr: exprs[index].take(),
                    });
                    index += 1;
                }
            }
            if all_literals {
                return original_expr;
            }

            Expr::Call(CallExpr {
                span,
                callee,
                args,
                type_args: None,
            })
        }
        other => other,
    }
}

fn dd_global_method_invocation<F>(span: Span, method: F) -> Callee
where
    F: FnOnce(Span) -> MemberProp,
{
    Callee::Expr(Box::new(Expr::Member(MemberExpr {
        span,
        prop: method(span),
        obj: Box::new(Expr::Member(MemberExpr {
            span,
            prop: MemberProp::Ident(Ident {
                span,
                sym: JsWord::from(DD_GLOBAL_NAMESPACE),
                optional: false,
            }),
            obj: Box::new(Expr::Ident(Ident {
                span,
                sym: JsWord::from(NODE_GLOBAL),
                optional: false,
            })),
        })),
    })))
}

fn get_plus_operator_based_on_num_of_args_for_span(arguments_len: usize, span: Span) -> Callee {
    match arguments_len {
        2 => return dd_global_method_invocation(span, two_items_plus_operator),
        3 => return dd_global_method_invocation(span, three_items_plus_operator),
        4 => return dd_global_method_invocation(span, four_items_plus_operator),
        5 => return dd_global_method_invocation(span, five_items_plus_operator),
        _other => dd_global_method_invocation(span, any_items_plus_operator),
    }
}

fn two_items_plus_operator(span: Span) -> MemberProp {
    MemberProp::Ident(Ident {
        span,
        sym: JsWord::from(DD_TWO_ITEMS_PLUS_OPERATOR),
        optional: false,
    })
}

fn three_items_plus_operator(span: Span) -> MemberProp {
    MemberProp::Ident(Ident {
        span,
        sym: JsWord::from(DD_THREE_ITEMS_PLUS_OPERATOR),
        optional: false,
    })
}

fn four_items_plus_operator(span: Span) -> MemberProp {
    MemberProp::Ident(Ident {
        span,
        sym: JsWord::from(DD_FOUR_ITEMS_PLUS_OPERATOR),
        optional: false,
    })
}

fn five_items_plus_operator(span: Span) -> MemberProp {
    MemberProp::Ident(Ident {
        span,
        sym: JsWord::from(DD_FIVE_ITEMS_PLUS_OPERATOR),
        optional: false,
    })
}

fn any_items_plus_operator(span: Span) -> MemberProp {
    MemberProp::Ident(Ident {
        span,
        sym: JsWord::from(DD_ANY_PLUS_OPERATOR),
        optional: false,
    })
}

fn template_literal_operator(span: Span) -> MemberProp {
    MemberProp::Ident(Ident {
        span,
        sym: JsWord::from("templateLiteralOperator"),
        optional: false,
    })
}
