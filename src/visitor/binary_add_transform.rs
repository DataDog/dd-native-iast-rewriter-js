/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use std::ops::DerefMut;
use swc::{
    common::{util::take::Take, Span},
    ecmascript::ast::*,
};

use super::{
    operation_transform_visitor::OperationTransformVisitor,
    visitor_util::get_dd_plus_operator_paren_expr,
};

pub struct BinaryAddTransform {}

impl BinaryAddTransform {
    pub fn to_dd_binary_expr(expr: &Expr, opv: &mut OperationTransformVisitor) -> Expr {
        let expr_clone = expr.clone();
        match expr_clone {
            Expr::Bin(mut binary) => {
                return to_dd_binary_expr_binary(&mut binary, opv);
            }
            _ => {}
        }
        expr_clone
    }
}

fn to_dd_binary_expr_binary(binary: &mut BinExpr, opv: &mut OperationTransformVisitor) -> Expr {
    let mut assignations = Vec::new();
    let mut arguments = Vec::new();
    if prepare_replace_expressions_in_binary(binary, &mut assignations, &mut arguments, opv) {
        return get_dd_plus_operator_paren_expr(
            Expr::Bin(binary.clone()),
            &arguments,
            &mut assignations,
            binary.span,
        );
    }
    Expr::Bin(binary.clone())
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

fn must_replace_binary_expression(arguments: &Vec<Expr>) -> bool {
    arguments.iter().any(|arg| match arg {
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
        Expr::Lit(_) => arguments.push(operand.clone()),

        Expr::Bin(binary) => {
            if binary.op != BinaryOp::Add {
                operand.map_with_mut(|op| {
                    Expr::Ident(opv.get_ident_used_in_assignation(
                        op,
                        assignations,
                        arguments,
                        span,
                    ))
                })
            } else {
                to_dd_binary_expr_binary(binary, opv);
            }
        }
        _ => operand.map_with_mut(|op| {
            Expr::Ident(opv.get_ident_used_in_assignation(op, assignations, arguments, span))
        }),
    }
}
