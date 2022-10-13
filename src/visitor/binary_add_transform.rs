/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use std::ops::DerefMut;
use swc::{
    common::{util::take::Take, Span},
    ecmascript::ast::*,
};

use super::{ident_provider::IdentProvider, visitor_util::get_dd_plus_operator_paren_expr};

pub struct BinaryAddTransform {}

impl BinaryAddTransform {
    pub fn to_dd_binary_expr(expr: &Expr, ident_provider: &mut dyn IdentProvider) -> Expr {
        let expr_clone = expr.clone();
        if let Expr::Bin(mut binary) = expr_clone {
            return to_dd_binary_expr_binary(&mut binary, ident_provider);
        }
        expr_clone
    }
}

fn to_dd_binary_expr_binary(binary: &mut BinExpr, ident_provider: &mut dyn IdentProvider) -> Expr {
    let mut assignations = Vec::new();
    let mut arguments = Vec::new();

    if prepare_replace_expressions_in_binary(
        binary,
        &mut assignations,
        &mut arguments,
        ident_provider,
    ) {
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
    assignations: &mut Vec<Expr>,
    arguments: &mut Vec<Expr>,
    ident_provider: &mut dyn IdentProvider,
) -> bool {
    let left = binary.left.deref_mut();
    replace_expressions_in_binary_operand(
        left,
        assignations,
        arguments,
        &binary.span,
        ident_provider,
    );

    let right = binary.right.deref_mut();
    replace_expressions_in_binary_operand(
        right,
        assignations,
        arguments,
        &binary.span,
        ident_provider,
    );

    // if all arguments are literals we can skip expression replacement
    must_replace_binary_expression(arguments)
}

fn must_replace_binary_expression(arguments: &[Expr]) -> bool {
    arguments.iter().any(|arg| !matches!(arg, Expr::Lit(_)))
}

fn replace_expressions_in_binary_operand(
    operand: &mut Expr,
    assignations: &mut Vec<Expr>,
    arguments: &mut Vec<Expr>,
    span: &Span,
    ident_provider: &mut dyn IdentProvider,
) {
    match operand {
        Expr::Lit(_) => arguments.push(operand.clone()),

        Expr::Bin(binary) => {
            if binary.op != BinaryOp::Add {
                operand.map_with_mut(|op| {
                    Expr::Ident(ident_provider.get_ident_used_in_assignation(
                        &op,
                        assignations,
                        arguments,
                        span,
                    ))
                })
            } else {
                to_dd_binary_expr_binary(binary, ident_provider);
            }
        }
        _ => operand.map_with_mut(|op| {
            Expr::Ident(ident_provider.get_ident_used_in_assignation(
                &op,
                assignations,
                arguments,
                span,
            ))
        }),
    }
}
