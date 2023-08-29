use serde::Serialize;
/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use std::collections::HashMap;
use swc::{
    common::{SourceFile, Span},
    ecmascript::ast::{Expr, ObjectLit, Program, Prop, VarDeclarator},
};
use swc_ecma_visit::{swc_ecma_ast::Lit, Visit, VisitMut, VisitMutWith};

#[derive(Serialize)]
pub struct HardcodedSecretResult {
    pub file: String,
    pub literals: Vec<LiteralInfo>,
}

#[derive(Serialize)]
pub struct LiteralInfo {
    pub value: String,
    pub ident: Option<String>,
    pub line: Option<usize>,
}

struct LiteralWithSpan {
    value: String,
    ident: Option<String>,
    span: Span,
}

pub struct HardcodedSecretVisitor {
    min_literal_length: usize,
    max_literal_length: usize,
    literals: HashMap<Span, LiteralWithSpan>,
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
                .values()
                .map(|literal| LiteralInfo {
                    value: literal.value.clone(),
                    ident: literal.ident.clone(),
                    line: source_file
                        .lookup_line(literal.span.lo)
                        .map(|line| line + 1),
                })
                .collect(),
        })
    }

    fn add_literal(&mut self, value: String, span: Span, ident: Option<String>) {
        if value.len() > self.min_literal_length
            && value.len() <= self.max_literal_length
            && !self.literals.contains_key(&span)
        {
            self.literals
                .insert(span, LiteralWithSpan { value, span, ident });
        }
    }
}

impl Visit for HardcodedSecretVisitor {}

impl VisitMut for HardcodedSecretVisitor {
    fn visit_mut_lit(&mut self, literal: &mut Lit) {
        if let Lit::Str(str_literal) = literal {
            let value = str_literal.value.to_string();
            self.add_literal(value, str_literal.span, None);
        }
    }

    fn visit_mut_var_declarators(&mut self, declarators: &mut Vec<VarDeclarator>) {
        declarators.iter_mut().for_each(|decl| {
            if let Some(decl_init) = decl.init.as_ref() {
                if let Expr::Lit(Lit::Str(str_literal)) = &**decl_init {
                    let ident = decl.name.as_ident().map(|ident| ident.id.sym.to_string());

                    let value = str_literal.value.to_string();
                    self.add_literal(value, str_literal.span, ident);
                }
            };
            decl.visit_mut_children_with(self);
        })
    }

    fn visit_mut_object_lit(&mut self, obj_lit: &mut ObjectLit) {
        let props = &obj_lit.props;
        props.iter().for_each(|prop_or_spread| {
            if let Some(prop) = prop_or_spread.as_prop() {
                if let Prop::KeyValue(key_value_prop) = &**prop {
                    if let Expr::Lit(Lit::Str(str_literal)) = &*key_value_prop.value {
                        let ident = key_value_prop
                            .key
                            .as_ident()
                            .map(|ident| ident.sym.to_string());

                        let value = str_literal.value.to_string();
                        self.add_literal(value, str_literal.span, ident);
                    }
                }
            }
        });
        obj_lit.visit_mut_children_with(self);
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
        program.visit_mut_with(&mut hardcoded_secret_visitor);
        hardcoded_secret_visitor.get_result(file, source_file)
    } else {
        None
    }
}
