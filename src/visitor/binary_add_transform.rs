/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use std::ops::DerefMut;
use swc::{
    atoms::JsWord,
    common::{util::take::Take, Span},
    ecmascript::ast::*,
};
use swc_ecma_visit::VisitMutWith;

use super::{
    operation_transform_visitor::OperationTransformVisitor,
    visitor_util::{get_dd_local_variable_name, get_plus_operator_based_on_num_of_args_for_span},
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
                    let plus_operator_call =
                        get_dd_call_plus_operator_expr(binary_clone, &arguments);

                    // if there are 0 assign expressions we can return just call expression without parentheses
                    // else wrap them all with a sequence of comma separated expressions inside parentheses
                    if assignations.len() == 0 {
                        return plus_operator_call;
                    } else {
                        let span = binary.span;
                        assignations.push(Box::new(plus_operator_call));
                        return Expr::Paren(ParenExpr {
                            span,
                            expr: Box::new(Expr::Seq(SeqExpr {
                                span,
                                exprs: assignations.clone(),
                            })),
                        });
                    }
                }
            }
            _ => {}
        }
        expr.clone()
    }
}

fn create_assign_expression(index: usize, expr: Expr, span: Span) -> (AssignExpr, Ident) {
    let id = Ident {
        span,
        sym: JsWord::from(get_dd_local_variable_name(index)),
        optional: false,
    };
    (
        AssignExpr {
            span,
            left: PatOrExpr::Pat(Box::new(Pat::Ident(BindingIdent {
                id: id.clone(),
                type_ann: None,
            }))),
            right: Box::new(expr),
            op: AssignOp::Assign,
        },
        id,
    )
}

fn must_replace_binary_expression(argument_exprs: &Vec<Expr>) -> bool {
    // only literals are filtered by now but may be other cases.
    argument_exprs.iter().any(|arg| match arg {
        Expr::Lit(_) => false,
        _ => true,
    })
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
        Expr::Call(_) | Expr::Paren(_) => {
            // visit_mut_children_with maybe only needed by Paren but...
            operand.visit_mut_children_with(opv);

            let (assign, id) = create_assign_expression(opv.next_ident(), operand.clone(), span);

            // store ident and assignation expression
            opv.idents.push(id.to_owned());

            assignations.push(Box::new(Expr::Assign(assign)));

            // store ident as argument
            arguments.push(Expr::Ident(id.clone()));

            // replace operand with new ident
            operand.map_with_mut(|_| Expr::Ident(id));
        }
        _ => arguments.push(operand.clone()),
    }
}

fn get_dd_call_plus_operator_expr(binary: BinExpr, arguments: &Vec<Expr>) -> Expr {
    let mut args: Vec<ExprOrSpread> = Vec::new();
    let span = binary.span;

    args.push(ExprOrSpread {
        expr: Box::new(Expr::Bin(binary)),
        spread: None,
    });

    args.append(
        &mut arguments
            .iter()
            .map(|expr| ExprOrSpread {
                expr: Box::new(expr.to_owned()),
                spread: None,
            })
            .collect::<Vec<_>>(),
    );

    Expr::Call(CallExpr {
        span,
        callee: get_plus_operator_based_on_num_of_args_for_span(args.len() - 1, span),
        args,
        type_args: None,
    })
}
