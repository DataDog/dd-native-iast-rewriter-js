/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use std::collections::HashSet;
use swc::atoms::JsWord;
use swc_common::{Span, SyntaxContext, DUMMY_SP};
use swc_ecma_ast::{
    AssignExpr, AssignOp, AssignTarget, BindingIdent, Expr, ExprOrSpread, Ident, SimpleAssignTarget,
};

use super::visitor_util::get_dd_local_variable_name;

pub trait IdentProvider {
    fn get_ident_used_in_assignation(
        &mut self,
        operand: &Expr,
        assignations: &mut Vec<Expr>,
        arguments: &mut Vec<ExprOrSpread>,
        span: &Span,
        is_spread: bool,
    ) -> Option<Ident> {
        let id = self.get_temporal_ident_used_in_assignation(operand, assignations, span);

        let id_expr = id
            .as_ref()
            .map_or_else(|| operand.clone(), |ident| Expr::Ident(ident.clone()));

        // store ident as argument
        let spread = if is_spread { Some(DUMMY_SP) } else { None };

        arguments.push(ExprOrSpread {
            spread,
            expr: Box::new(id_expr.clone()),
        });

        id
    }

    fn get_temporal_ident_used_in_assignation(
        &mut self,
        operand: &Expr,
        assignations: &mut Vec<Expr>,
        span: &Span,
    ) -> Option<Ident> {
        if operand.is_lit() {
            return None;
        }

        let next_ident = self.next_ident();
        let (assign, id) = self.create_assign_expression(next_ident, operand, span);

        // store ident and assignation expression
        self.register_ident(id.clone());

        assignations.push(Expr::Assign(assign));

        Some(id)
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
            ctxt: SyntaxContext::empty(),
        };
        (
            AssignExpr {
                span: *span,
                left: AssignTarget::Simple(SimpleAssignTarget::Ident(BindingIdent {
                    id: id.clone(),
                    type_ann: None,
                })),
                right: Box::new(expr.clone()),
                op: AssignOp::Assign,
            },
            id,
        )
    }

    fn register_ident(&mut self, ident: Ident);

    fn next_ident(&mut self) -> usize;

    fn get_local_var_prefix(&mut self) -> String;

    fn reset_counter(&mut self);

    fn register_variable(&mut self, variable: &Ident);
}

pub struct DefaultIdentProvider {
    pub ident_counter: usize,
    pub idents: Vec<Ident>,
    pub variable_decl: HashSet<Ident>,
    pub local_var_prefix: String,
}

impl DefaultIdentProvider {
    pub fn new(local_var_prefix: &str) -> DefaultIdentProvider {
        DefaultIdentProvider {
            ident_counter: 0,
            idents: Vec::new(),
            variable_decl: HashSet::new(),
            local_var_prefix: local_var_prefix.to_string(),
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

    fn reset_counter(&mut self) {
        self.ident_counter = 0;
    }

    fn register_variable(&mut self, variable: &Ident) {
        self.variable_decl.insert(variable.clone());
    }

    fn get_local_var_prefix(&mut self) -> String {
        self.local_var_prefix.clone()
    }
}
