/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use swc_common::{util::take::Take, Span};
use swc_ecma_ast::ExprOrSpread;
use swc_ecma_visit::swc_ecma_ast::{BinaryOp, Expr};

use crate::visitor::ident_provider::IdentProvider;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum IdentMode {
    Replace,
    Keep,
}

pub trait OperandHandler {
    fn replace_expressions_in_expr_or_spread(
        operand: &mut ExprOrSpread,
        ident_mode: IdentMode,
        assignations: &mut Vec<Expr>,
        arguments: &mut Vec<ExprOrSpread>,
        span: &Span,
        ident_provider: &mut dyn IdentProvider,
        expand_arrays: bool,
    ) {
        let is_spread = operand.spread.is_some();
        let operand_expr = &mut *operand.expr;

        Self::replace_expressions_in_expr(
            operand_expr,
            ident_mode,
            assignations,
            arguments,
            span,
            ident_provider,
            is_spread,
            expand_arrays,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn replace_expressions_in_expr(
        expr: &mut Expr,
        ident_mode: IdentMode,
        assignations: &mut Vec<Expr>,
        arguments: &mut Vec<ExprOrSpread>,
        span: &Span,
        ident_provider: &mut dyn IdentProvider,
        is_spread: bool,
        expand_arrays: bool,
    ) {
        match expr {
            Expr::Lit(_) => Self::replace_literals(expr, arguments),
            Expr::Ident(_) => {
                if ident_mode == IdentMode::Replace {
                    expr.map_with_mut(|op| {
                        ident_provider
                            .get_ident_used_in_assignation(
                                &op,
                                assignations,
                                arguments,
                                span,
                                is_spread,
                            )
                            .map_or(op, Expr::Ident)
                    })
                } else {
                    arguments.push(ExprOrSpread::from(expr.clone()))
                }
            }
            Expr::Bin(ref binary) => Self::replace_binary(
                binary.op,
                expr,
                assignations,
                arguments,
                span,
                ident_provider,
                is_spread,
            ),
            Expr::Array(array) if expand_arrays => array.elems.iter_mut().for_each(|elem_opt| {
                if elem_opt.is_some() {
                    let elem = elem_opt.as_mut().unwrap();
                    Self::replace_expressions_in_expr_or_spread(
                        elem,
                        ident_mode,
                        assignations,
                        arguments,
                        span,
                        ident_provider,
                        false,
                    )
                }
            }),
            _ => Self::replace_default(
                expr,
                assignations,
                arguments,
                span,
                ident_provider,
                is_spread,
            ),
        }
    }

    fn get_ident_mode(operand: &mut Expr) -> IdentMode {
        if operand.is_ident() || operand.is_lit() {
            IdentMode::Keep
        } else {
            IdentMode::Replace
        }
    }

    fn replace_literals(operand: &mut Expr, arguments: &mut Vec<ExprOrSpread>) {
        arguments.push(ExprOrSpread::from(operand.clone()))
    }

    fn replace_default(
        operand: &mut Expr,
        assignations: &mut Vec<Expr>,
        arguments: &mut Vec<ExprOrSpread>,
        span: &Span,
        ident_provider: &mut dyn IdentProvider,
        is_spread: bool,
    ) {
        operand.map_with_mut(|op| {
            ident_provider
                .get_ident_used_in_assignation(&op, assignations, arguments, span, is_spread)
                .map_or(op, Expr::Ident)
        })
    }

    fn replace_binary(
        binary_op: BinaryOp,
        operand: &mut Expr,
        assignations: &mut Vec<Expr>,
        arguments: &mut Vec<ExprOrSpread>,
        span: &Span,
        ident_provider: &mut dyn IdentProvider,
        is_spread: bool,
    ) {
        if binary_op != BinaryOp::Add {
            Self::replace_default(
                operand,
                assignations,
                arguments,
                span,
                ident_provider,
                is_spread,
            );
        }
    }
}

pub struct DefaultOperandHandler {}
impl OperandHandler for DefaultOperandHandler {}
