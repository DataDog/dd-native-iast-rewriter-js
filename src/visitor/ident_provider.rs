use std::collections::HashSet;

/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use swc::{
    atoms::JsWord,
    common::{Span, DUMMY_SP},
};
use swc_ecma_visit::swc_ecma_ast::{
    AssignExpr, AssignOp, BindingIdent, Expr, Ident, Pat, PatOrExpr,
};

use super::{transform_status::TransformStatus, visitor_util::get_dd_local_variable_name};

pub trait IdentProvider {
    fn get_ident_used_in_assignation(
        &mut self,
        operand: &Expr,
        assignations: &mut Vec<Expr>,
        arguments: &mut Vec<Expr>,
        span: &Span,
    ) -> Ident {
        let id = self.get_temporal_ident_used_in_assignation(operand, assignations, span);

        // store ident as argument
        arguments.push(Expr::Ident(id.clone()));

        id
    }

    fn get_temporal_ident_used_in_assignation(
        &mut self,
        operand: &Expr,
        assignations: &mut Vec<Expr>,
        span: &Span,
    ) -> Ident {
        let next_ident = self.next_ident();
        let (assign, id) = self.create_assign_expression(next_ident, operand, span);

        // store ident and assignation expression
        self.register_ident(id.clone());

        assignations.push(Expr::Assign(assign));

        id
    }

    fn create_assign_expression(
        &mut self,
        index: usize,
        expr: &Expr,
        span: &Span,
    ) -> (AssignExpr, Ident) {
        let id = Ident {
            span: DUMMY_SP,
            sym: JsWord::from(get_dd_local_variable_name(
                index,
                &self.get_local_var_prefix(),
            )),
            optional: false,
        };
        (
            AssignExpr {
                span: *span,
                left: PatOrExpr::Pat(Box::new(Pat::Ident(BindingIdent {
                    id: id.clone(),
                    type_ann: None,
                }))),
                right: Box::new(expr.clone()),
                op: AssignOp::Assign,
            },
            id,
        )
    }

    fn register_ident(&mut self, ident: Ident);

    fn next_ident(&mut self) -> usize;

    fn set_status(&mut self, status: TransformStatus);

    fn get_local_var_prefix(&mut self) -> String;
    fn reset_counter(&mut self);

    fn register_variable(&mut self, variable: &Ident);
}

pub struct DefaultIdentProvider {
    pub ident_counter: usize,
    pub idents: Vec<Ident>,
    pub variable_decl: HashSet<Ident>,
    pub transform_status: TransformStatus,
}

impl DefaultIdentProvider {
    pub fn new() -> Self {
        DefaultIdentProvider {
            ident_counter: 0,
            idents: Vec::new(),
            variable_decl: HashSet::new(),
            transform_status: TransformStatus::not_modified(),
        }
    }
}

impl IdentProvider for DefaultIdentProvider {
    fn register_ident(&mut self, ident: Ident) {
        if !self.idents.contains(&ident) {
            self.idents.push(ident);
        }
    }

    fn next_ident(&mut self) -> usize {
        let counter = self.ident_counter;
        self.ident_counter += 1;
        counter
    }

    fn set_status(&mut self, status: TransformStatus) {
        self.transform_status = status;
    }

    fn reset_counter(&mut self) {
        self.ident_counter = 0;
    }

    fn register_variable(&mut self, variable: &Ident) {
        self.variable_decl.insert(variable.clone());
    }
}
