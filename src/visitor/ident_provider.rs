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
        self.get_ident_used_in_assignation_with_definitive(
            operand,
            assignations,
            arguments,
            span,
            true,
        )
    }

    fn get_ident_used_in_assignation_with_definitive(
        &mut self,
        operand: &Expr,
        assignations: &mut Vec<Expr>,
        arguments: &mut Vec<Expr>,
        span: &Span,
        definitive: bool,
    ) -> Ident {
        let next_ident = self.next_ident();
        let (assign, id) = self.create_assign_expression(next_ident, operand, span);

        // store ident and assignation expression
        let id_clone = id.clone();
        if definitive {
            self.register_ident(id_clone);
        }

        assignations.push(Expr::Assign(assign));

        // store ident as argument
        arguments.push(Expr::Ident(id.clone()));

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
}
