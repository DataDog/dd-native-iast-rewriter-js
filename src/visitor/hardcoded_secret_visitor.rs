/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use swc::{
    common::{SourceFile, Span},
    ecmascript::ast::{Callee, Expr, ObjectLit, Program, Prop, Str, VarDeclarator},
};
use swc_ecma_visit::{swc_ecma_ast::Lit, Visit, VisitWith};

#[derive(Serialize)]
pub struct HardcodedSecretResult {
    pub file: String,
    pub literals: Vec<LiteralInfo>,
}

#[derive(Serialize)]
pub struct LiteralLocation {
    pub ident: Option<String>,
    pub line: Option<usize>,
}

#[derive(Serialize)]
pub struct LiteralInfo {
    pub value: String,
    pub locations: Vec<LiteralLocation>,
}

#[derive(Eq, Hash, PartialEq)]
struct SpanAndIdent {
    span: Span,
    ident: Option<String>,
}

pub struct HardcodedSecretVisitor {
    min_literal_length: usize,
    max_literal_length: usize,
    literals: HashMap<String, HashSet<SpanAndIdent>>,
}

impl HardcodedSecretVisitor {
    pub fn default() -> Self {
        HardcodedSecretVisitor {
            min_literal_length: 10,
            max_literal_length: 256,
            literals: HashMap::new(),
        }
    }

    pub fn get_result(
        &self,
        file: &str,
        source_file: &SourceFile,
    ) -> Option<HardcodedSecretResult> {
        Some(HardcodedSecretResult {
            file: file.to_owned(),
            literals: self
                .literals
                .iter()
                .map(|(value, literal_spans)| LiteralInfo {
                    value: value.clone(),
                    locations: literal_spans
                        .iter()
                        .map(|literal_span| LiteralLocation {
                            ident: literal_span.ident.clone(),
                            line: source_file
                                .lookup_line(literal_span.span.lo)
                                .map(|line| line + 1),
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

            self.literals.get_mut(&value).and_then(|literal_spans| {
                // check if the span has been inserted before and skip it if true
                if !literal_spans
                    .iter()
                    .any(|literal_span| literal_span.span == span)
                {
                    Some(literal_spans.insert(SpanAndIdent { span, ident }))
                } else {
                    None
                }
            });
        }
    }
}

impl Visit for HardcodedSecretVisitor {
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
                        if ident.sym.to_string() == "require"
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
                    if ident.sym.to_string() == "RegExp"
                        && new_exp
                            .args
                            .as_ref()
                            .map(|args| args[0].spread.is_none() && args[0].expr.is_lit())
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

pub fn get_hardcoded_secrets(
    hardcoded_secret_enabled: bool,
    file: &str,
    source_file: &SourceFile,
    program: &mut Program,
) -> Option<HardcodedSecretResult> {
    if hardcoded_secret_enabled {
        let mut hardcoded_secret_visitor = HardcodedSecretVisitor::default();
        program.visit_with(&mut hardcoded_secret_visitor);
        hardcoded_secret_visitor.get_result(file, source_file)
    } else {
        None
    }
}
