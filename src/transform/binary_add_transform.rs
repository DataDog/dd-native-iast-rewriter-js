/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use swc::ecmascript::ast::*;

use crate::{
    transform::operand_handler::{DefaultOperandHandler, OperandHandler},
    visitor::{
        csi_methods::CsiMethods, ident_provider::IdentProvider, transform_status::TransformStatus,
        visitor_util::get_dd_paren_expr,
    },
};

pub struct BinaryAddTransform {}

impl BinaryAddTransform {
    pub fn to_dd_binary_expr(
        expr: &Expr,
        csi_methods: &CsiMethods,
        ident_provider: &mut dyn IdentProvider,
    ) -> Expr {
        let expr_clone = expr.clone();
        if let Expr::Bin(mut binary) = expr_clone {
            return to_dd_binary_expr_binary(&mut binary, csi_methods, ident_provider);
        }
        expr_clone
    }
}

fn to_dd_binary_expr_binary(
    binary: &mut BinExpr,
    csi_methods: &CsiMethods,
    ident_provider: &mut dyn IdentProvider,
) -> Expr {
    let mut assignations = Vec::new();
    let mut arguments = Vec::new();

    if prepare_replace_expressions_in_binary(
        binary,
        &mut assignations,
        &mut arguments,
        ident_provider,
    ) {
        ident_provider.set_status(TransformStatus::modified());
        return get_dd_paren_expr(
            &Expr::Bin(binary.clone()),
            &arguments,
            &mut assignations,
            &csi_methods.get_dd_plus_operator_name(),
            &binary.span,
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
    let left_ident_mode = DefaultOperandHandler::get_ident_mode(&mut *binary.right);
    DefaultOperandHandler::replace_expressions_in_operand(
        &mut *binary.left,
        left_ident_mode,
        assignations,
        arguments,
        &binary.span,
        ident_provider,
    );

    let right_ident_mode = DefaultOperandHandler::get_ident_mode(&mut *binary.left);
    DefaultOperandHandler::replace_expressions_in_operand(
        &mut *binary.right,
        right_ident_mode,
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
