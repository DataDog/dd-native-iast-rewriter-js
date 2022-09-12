/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use std::ops::DerefMut;
use swc::{
    common::{util::take::Take, Span},
    ecmascript::ast::*,
};
use swc_ecma_visit::VisitMutWith;

use crate::visitor::visitor_util::is_typeof;

use super::{
    operation_transform_visitor::OperationTransformVisitor,
    visitor_util::get_dd_plus_operator_paren_expr,
};

pub struct BinaryAddTransform {}

impl BinaryAddTransform {
    pub fn to_dd_binary_expr(expr: &Expr, opv: &mut OperationTransformVisitor) -> Expr {
        match expr {
            Expr::Bin(binary) => {
                let mut binary_clone = binary.clone();

                let mut assignations = Vec::new();
                let mut arguments = Vec::new();
                if prepare_replace_expressions_in_binary(
                    &mut binary_clone,
                    &mut assignations,
                    &mut arguments,
                    opv,
                ) {
                    return get_dd_plus_operator_paren_expr(
                        Expr::Bin(binary_clone),
                        &arguments,
                        &mut assignations,
                        binary.span,
                    );
                }
            }
            _ => {}
        }
        expr.clone()
    }
}

fn prepare_replace_expressions_in_binary(
    binary: &mut BinExpr,
    assignations: &mut Vec<Box<Expr>>,
    arguments: &mut Vec<Expr>,
    opv: &mut OperationTransformVisitor,
) -> bool {
    let left = binary.left.deref_mut();
    replace_expressions_in_binary_operand(left, assignations, arguments, binary.span, opv);

    let right = binary.right.deref_mut();
    replace_expressions_in_binary_operand(right, assignations, arguments, binary.span, opv);

    // if all arguments are literals we can skip expression replacement
    return must_replace_binary_expression(&arguments);
}

fn must_replace_binary_expression(argument_exprs: &Vec<Expr>) -> bool {
    // only literals are filtered by now but may be other cases.
    argument_exprs.iter().any(|arg| match arg {
        Expr::Lit(_) => false,
        _ => true,
    })
}

fn replace_expressions_in_binary_operand(
    operand: &mut Expr,
    assignations: &mut Vec<Box<Expr>>,
    arguments: &mut Vec<Expr>,
    span: Span,
    opv: &mut OperationTransformVisitor,
) {
    match operand {
        Expr::Bin(binary) => {
            if binary.op == BinaryOp::Add {
                prepare_replace_expressions_in_binary(binary, assignations, arguments, opv);
            } else {
                arguments.push(operand.clone())
            }
        }
        Expr::Call(_) | Expr::Paren(_) | Expr::Tpl(_) | Expr::Await(_) => {
            operand.visit_mut_children_with(opv);

            operand.map_with_mut(|op| {
                Expr::Ident(opv.get_ident_used_in_assignation(op, assignations, arguments, span))
            })
        }
        Expr::Member(_) => operand.map_with_mut(|op| {
            Expr::Ident(opv.get_ident_used_in_assignation(op, assignations, arguments, span))
        }),
        Expr::Unary(unary) => {
            if is_typeof(&unary) {
                operand.map_with_mut(|op| {
                    Expr::Ident(opv.get_ident_used_in_assignation(
                        op,
                        assignations,
                        arguments,
                        span,
                    ))
                })
            }
        }
        _ => arguments.push(operand.clone()),
    }
}
