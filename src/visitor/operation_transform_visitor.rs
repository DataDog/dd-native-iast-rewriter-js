/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use std::collections::HashSet;
use swc::{common::util::take::Take, ecmascript::ast::*};
use swc_ecma_visit::{Visit, VisitMut, VisitMutWith};

use super::{
    assign_add_transform::AssignAddTransform, binary_add_transform::BinaryAddTransform,
    template_transform::TemplateTransform,
};

pub struct OperationTransformVisitor {
    pub ident_counter: usize,
    pub idents: Vec<Ident>,
    pub variable_decl: HashSet<Ident>,
}

impl OperationTransformVisitor {
    pub fn new() -> Self {
        OperationTransformVisitor {
            ident_counter: 0,
            idents: Vec::new(),
            variable_decl: HashSet::new(),
        }
    }

    pub fn next_ident(&mut self) -> usize {
        let counter = self.ident_counter;
        self.ident_counter += 1;
        return counter;
    }
}

impl Visit for OperationTransformVisitor {}

impl VisitMut for OperationTransformVisitor {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Bin(binary) => {
                if binary.op == BinaryOp::Add {
                    expr.map_with_mut(|bin| BinaryAddTransform::to_dd_binary_expr(&bin, self));
                } else {
                    expr.visit_mut_children_with(self);
                }
            }
            Expr::Assign(assign) => {
                assign.visit_mut_children_with(self);
                assign.map_with_mut(|assign| AssignAddTransform::to_dd_assign_expr(assign));
            }
            Expr::Tpl(tpl) => {
                if !tpl.exprs.is_empty() {
                    tpl.exprs.visit_mut_children_with(self);
                    expr.map_with_mut(|tpl| TemplateTransform::to_dd_tpl_expr(&tpl, self));
                }
            }
            _ => {
                expr.visit_mut_children_with(self);
            }
        }
    }

    fn visit_mut_ident(&mut self, ident: &mut Ident) {
        self.variable_decl.insert(ident.to_owned());
    }

    fn visit_mut_if_stmt(&mut self, if_stmt: &mut IfStmt) {
        if_stmt.test.visit_mut_children_with(self);
        if_stmt.cons.visit_mut_children_with(self);
    }

    // cancel visit child blocks
    fn visit_mut_block_stmt(&mut self, _n: &mut BlockStmt) {}
}
