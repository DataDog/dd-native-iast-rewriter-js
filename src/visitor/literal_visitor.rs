use log::debug;
/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use swc::Compiler;
use swc_common::Span;
use swc_ecma_ast::{Callee, Expr, ObjectLit, Program, Prop, Str, VarDeclarator};
use swc_ecma_visit::{swc_ecma_ast::Lit, Visit, VisitWith};

#[derive(Serialize)]
pub struct LiteralsResult {
    pub file: String,
    pub literals: Vec<LiteralInfo>,
}

#[derive(Serialize)]
pub struct LiteralLocation {
    pub ident: Option<String>,
    pub line: usize,
    pub column: usize,
}

#[derive(Serialize)]
pub struct LiteralInfo {
    pub value: String,
    pub locations: Vec<LiteralLocation>,
}

#[derive(Eq)]
struct SpanAndIdent {
    span: Span,
    ident: Option<String>,
}

impl PartialEq for SpanAndIdent {
    fn eq(&self, other: &Self) -> bool {
        self.span == other.span
    }
}

impl std::hash::Hash for SpanAndIdent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.span.hash(state);
    }
}

pub struct LiteralVisitor {
    min_literal_length: usize,
    max_literal_length: usize,
    literals: HashMap<String, HashSet<SpanAndIdent>>,
}

impl LiteralVisitor {
    pub fn default() -> Self {
        LiteralVisitor {
            min_literal_length: 10,
            max_literal_length: 256,
            literals: HashMap::new(),
        }
    }

    pub fn get_result(&self, file: &str, compiler: &Compiler) -> Option<LiteralsResult> {
        Some(LiteralsResult {
            file: file.to_owned(),
            literals: self
                .literals
                .iter()
                .map(|(value, spans)| LiteralInfo {
                    value: value.clone(),
                    locations: spans
                        .iter()
                        .map(|span_and_ident| {
                            let pos = compiler.cm.lookup_char_pos(span_and_ident.span.lo);
                            LiteralLocation {
                                ident: span_and_ident.ident.clone(),
                                line: pos.line,
                                column: pos.col.0 + 1,
                            }
                        })
                        .collect(),
                })
                .collect(),
        })
    }

    fn add_literal(&mut self, str_literal: &Str, ident: Option<String>) {
        let value = str_literal.value.to_string();
        let span = str_literal.span;

        if value.len() > self.min_literal_length && value.len() <= self.max_literal_length {
            if !self.literals.contains_key(&value) {
                self.literals.insert(value.clone(), HashSet::new());
            }

            self.literals
                .get_mut(&value)
                .map(|spans| spans.insert(SpanAndIdent { span, ident }));
        }
    }
}

impl Visit for LiteralVisitor {
    fn visit_lit(&mut self, literal: &Lit) {
        if let Lit::Str(str_literal) = literal {
            self.add_literal(str_literal, None);
        }
    }

    fn visit_var_declarators(&mut self, declarators: &[VarDeclarator]) {
        declarators.iter().for_each(|decl| {
            if let Some(decl_init) = decl.init.as_ref() {
                if let Expr::Lit(Lit::Str(str_literal)) = &**decl_init {
                    let ident = decl.name.as_ident().map(|ident| ident.id.sym.to_string());

                    self.add_literal(str_literal, ident);
                }
            };
            decl.visit_children_with(self);
        })
    }

    fn visit_object_lit(&mut self, obj_lit: &ObjectLit) {
        let props = &obj_lit.props;
        props.iter().for_each(|prop_or_spread| {
            if let Some(prop) = prop_or_spread.as_prop() {
                if let Prop::KeyValue(key_value_prop) = &**prop {
                    if let Expr::Lit(Lit::Str(str_literal)) = &*key_value_prop.value {
                        let ident = key_value_prop
                            .key
                            .as_ident()
                            .map(|ident| ident.sym.to_string());

                        self.add_literal(str_literal, ident);
                    }
                }
            }
        });
        obj_lit.visit_children_with(self);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Call(call) => {
                if let Callee::Expr(callee_expr) = &call.callee {
                    if let Expr::Ident(ident) = &**callee_expr {
                        if ident.sym == "require"
                            && !call.args.is_empty()
                            && call.args[0].spread.is_none()
                            && call.args[0].expr.is_lit()
                        {
                            // if the call is a require('path_or_module') skip visiting children
                            return;
                        }
                    }
                }
            }

            Expr::New(new_exp) => {
                if let Expr::Ident(ident) = &*new_exp.callee {
                    if ident.sym == "RegExp"
                        && new_exp
                            .args
                            .as_ref()
                            .map(|args| {
                                !args.is_empty()
                                    && args[0].spread.is_none()
                                    && args[0].expr.is_lit()
                            })
                            .is_some()
                    {
                        // if the call is a new RegExp('regex') skip visiting children
                        return;
                    }
                }
            }

            _ => {}
        }

        expr.visit_children_with(self);
    }
}

pub fn get_literals(
    literals_enabled: bool,
    file: &str,
    program: &mut Program,
    compiler: &Compiler,
) -> Option<LiteralsResult> {
    if literals_enabled {
        debug!("Searching for literals");

        let mut literal_visitor = LiteralVisitor::default();
        program.visit_with(&mut literal_visitor);

        literal_visitor.get_result(file, compiler)
    } else {
        None
    }
}
