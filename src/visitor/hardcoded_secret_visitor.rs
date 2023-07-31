/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use swc_ecma_visit::swc_ecma_ast::Lit;

pub trait HardcodedSecretVisitor {
    fn visit_lit(&mut self, literal: &Lit);

    fn get_found_literals(&self) -> Vec<String>;
}

pub struct DefaultHardcodedSecretVisitor {
    min_literal_length: usize,
    literals: Vec<String>,
}

impl HardcodedSecretVisitor for DefaultHardcodedSecretVisitor {
    fn visit_lit(&mut self, literal: &Lit) {
        if let Lit::Str(str_literal) = literal {
            let value = str_literal.value.to_string();
            if value.len() > self.min_literal_length {
                self.literals.push(value);
            }
        }
    }

    fn get_found_literals(&self) -> Vec<String> {
        self.literals.clone()
    }
}

pub struct NoOpHardcodedSecretVisitor {}

impl HardcodedSecretVisitor for NoOpHardcodedSecretVisitor {
    fn visit_lit(&mut self, _literal: &Lit) {}

    fn get_found_literals(&self) -> Vec<String> {
        vec![]
    }
}

pub struct HardcodedSecretResult {
    pub matches: Vec<String>,
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

pub fn get_hardcoded_secret_matches(literals: &[String]) -> HardcodedSecretResult {
    // TODO: run regexes
    HardcodedSecretResult {
        matches: literals.to_vec(),
    }
}
