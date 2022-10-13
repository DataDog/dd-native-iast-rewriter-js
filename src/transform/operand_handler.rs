/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use swc::common::{util::take::Take, Span};
use swc_ecma_visit::swc_ecma_ast::{BinaryOp, Expr};

use crate::visitor::ident_provider::IdentProvider;

pub trait OperandHandler {
    fn replace_expressions_in_operand(
        operand: &mut Expr,
        assignations: &mut Vec<Expr>,
        arguments: &mut Vec<Expr>,
        span: &Span,
        ident_provider: &mut dyn IdentProvider,
    ) {
        match operand {
            Expr::Lit(_) => Self::replace_literals(operand, arguments),
            Expr::Bin(ref binary) => Self::replace_binary(
                binary.op,
                operand,
                assignations,
                arguments,
                span,
                ident_provider,
            ),
            _ => Self::replace_default(operand, assignations, arguments, span, ident_provider),
        }
    }

    fn replace_literals(operand: &mut Expr, arguments: &mut Vec<Expr>) {
        arguments.push(operand.clone())
    }

    fn replace_default(
        operand: &mut Expr,
        assignations: &mut Vec<Expr>,
        arguments: &mut Vec<Expr>,
        span: &Span,
        ident_provider: &mut dyn IdentProvider,
    ) {
        operand.map_with_mut(|op| {
            Expr::Ident(ident_provider.get_ident_used_in_assignation(
                &op,
                assignations,
                arguments,
                span,
            ))
        })
    }

    fn replace_binary(
        binary_op: BinaryOp,
        operand: &mut Expr,
        assignations: &mut Vec<Expr>,
        arguments: &mut Vec<Expr>,
        span: &Span,
        ident_provider: &mut dyn IdentProvider,
    ) {
        if binary_op != BinaryOp::Add {
            Self::replace_default(operand, assignations, arguments, span, ident_provider);
        }
    }
}

pub struct DefaultOperandHandler {}
impl OperandHandler for DefaultOperandHandler {}
