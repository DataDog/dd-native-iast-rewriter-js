/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use serde::Serialize;
use swc::common::{SourceFile, Span};
use swc_ecma_visit::swc_ecma_ast::Lit;

pub trait HardcodedSecretVisitor {
    fn visit_lit(&mut self, literal: &Lit);

    fn get_result(&self, file: &str, source_file: &SourceFile) -> Option<HardcodedSecretResult>;
}

struct LiteralWithSpan {
    value: String,
    span: Span,
}

pub struct DefaultHardcodedSecretVisitor {
    min_literal_length: usize,
    literals: Vec<LiteralWithSpan>,
}

impl HardcodedSecretVisitor for DefaultHardcodedSecretVisitor {
    fn visit_lit(&mut self, literal: &Lit) {
        if let Lit::Str(str_literal) = literal {
            let value = str_literal.value.to_string();
            if value.len() > self.min_literal_length {
                self.literals.push(LiteralWithSpan {
                    value,
                    span: str_literal.span,
                });
            }
        }
    }

    fn get_result(&self, file: &str, source_file: &SourceFile) -> Option<HardcodedSecretResult> {
        Some(HardcodedSecretResult {
            file: file.to_owned(),
            literals: self
                .literals
                .as_slice()
                .iter()
                .map(|literal| LiteralInfo {
                    value: literal.value.clone(),
                    line: source_file
                        .lookup_line(literal.span.lo)
                        .map(|line| line + 1),
                })
                .collect(),
        })
    }
}

pub struct NoOpHardcodedSecretVisitor {}

impl HardcodedSecretVisitor for NoOpHardcodedSecretVisitor {
    fn visit_lit(&mut self, _literal: &Lit) {}

    fn get_result(&self, _file: &str, _source_file: &SourceFile) -> Option<HardcodedSecretResult> {
        None
    }
}

#[derive(Serialize)]
pub struct HardcodedSecretResult {
    pub file: String,
    pub literals: Vec<LiteralInfo>,
}

#[derive(Serialize)]
pub struct LiteralInfo {
    pub value: String,
    pub line: Option<usize>,
}

pub fn get_hardcoded_secret_visitor(enabled: bool) -> Box<dyn HardcodedSecretVisitor> {
    if enabled {
        Box::new(DefaultHardcodedSecretVisitor {
            min_literal_length: 8,
            literals: vec![],
        })
    } else {
        Box::new(NoOpHardcodedSecretVisitor {})
    }
}
