/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use std::collections::HashSet;
use swc::atoms::JsWord;
use swc_common::{Span, SyntaxContext, DUMMY_SP};
use swc_ecma_ast::{
    ArrayLit, AssignExpr, AssignOp, AssignTarget, BindingIdent, Expr, ExprOrSpread, Ident,
    SimpleAssignTarget,
};

use super::visitor_util::get_dd_local_variable_name;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum IdentKind {
    Expr,
    Spread,
}

impl From<ExprOrSpread> for IdentKind {
    fn from(expr_or_spread: ExprOrSpread) -> Self {
        expr_or_spread
            .spread
            .map_or_else(|| IdentKind::Expr, |_| IdentKind::Spread)
    }
}

pub trait IdentProvider {
    fn get_ident_used_in_assignation(
        &mut self,
        operand: &Expr,
        assignations: &mut Vec<Expr>,
        arguments: &mut Vec<ExprOrSpread>,
        span: &Span,
        ident_kind: IdentKind,
    ) -> Option<Ident> {
        let id =
            self.get_temporal_ident_used_in_assignation(operand, assignations, span, ident_kind);

        let id_expr = id
            .as_ref()
            .map_or_else(|| operand.clone(), |ident| Expr::Ident(ident.clone()));

        // store ident as argument
        let spread = if ident_kind == IdentKind::Spread {
            Some(DUMMY_SP)
        } else {
            None
        };

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
        ident_kind: IdentKind,
    ) -> Option<Ident> {
        if operand.is_lit() {
            return None;
        }

        let next_ident = self.next_ident();
        let (assign, id) = self.create_assign_expression(next_ident, operand, span, ident_kind);

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
        ident_kind: IdentKind,
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
                right: self.create_assign_right_operand_expression(expr, ident_kind),
                op: AssignOp::Assign,
            },
            id,
        )
    }

    fn create_assign_right_operand_expression(
        &mut self,
        expr: &Expr,
        ident_kind: IdentKind,
    ) -> Box<Expr> {
        // when is_spread, create a new array with the spread expression [...a] to avoid spreading it twice.
        // 'a' could be a Proxy which intercepts the get and does some operation on every call
        // (__datadog_test_0 = "hello".concat, __datadog_test_1 = [...a], _ddiast.concat(__datadog_test_0.call("hello", ...__datadog_test_1), __datadog_test_0, "hello", ...__datadog_test_1))
        let right_ep = if ident_kind == IdentKind::Spread {
            Expr::Array(ArrayLit {
                span: DUMMY_SP,
                elems: vec![Some(ExprOrSpread {
                    spread: Some(DUMMY_SP),
                    expr: Box::new(expr.clone()),
                })],
            })
        } else {
            expr.clone()
        };

        Box::new(right_ep)
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
