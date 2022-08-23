use swc::{common::util::take::Take, ecmascript::ast::*};
use swc_ecma_visit::VisitMut;

use crate::{
    assign_transform_visitor::AssignOp::{AddAssign, Assign},
    visitor_util::{get_plus_operator_based_on_num_of_args_for_span, is_dd_method},
};

pub struct AssignTransformVisitor {}

impl VisitMut for AssignTransformVisitor {
    fn visit_mut_assign_expr(&mut self, assign: &mut AssignExpr) {
        if assign.op == AddAssign {
            assign.map_with_mut(|assign| to_dd_assign_expr(assign));
        }
    }
}

fn to_dd_assign_expr(assign: AssignExpr) -> AssignExpr {
    let span = assign.span;
    let op = assign.op;
    let left = assign.left;
    let mut right = assign.right;

    return match left {
        PatOrExpr::Pat(_) => AssignExpr {
            span,
            op,
            left,
            right,
        },
        PatOrExpr::Expr(left_expr) => {
            let left = *left_expr;

            let mut args = vec![ExprOrSpread {
                spread: None,
                expr: Box::new(left.clone()),
            }];

            // if a += global._ddiast.twoItemsPlusOperator(b + c, b, c) then convert it to a = global._ddiast.threeItemsPlusOperator(a + b + c, a, b, c)
            if right_is_a_call_to_dd_method(&right) {
                right = Box::new(extract_call_arguments(&right, &mut args));
            } else {
                args.push(ExprOrSpread {
                    spread: None,
                    expr: right.clone(),
                })
            }

            args.insert(
                0,
                ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Bin(BinExpr {
                        span,
                        op: BinaryOp::Add,
                        left: Box::new(left.clone()),
                        right: right.clone(),
                    })),
                },
            );

            let binary = Expr::Call(CallExpr {
                span,
                callee: get_plus_operator_based_on_num_of_args_for_span(args.len() - 1, span),
                args,
                type_args: None,
            });
            return AssignExpr {
                span,
                op: Assign,
                left: PatOrExpr::Expr(Box::new(left)),
                right: Box::new(binary),
            };
        }
    };
}

fn right_is_a_call_to_dd_method(right: &Expr) -> bool {
    match right {
        Expr::Call(call) => is_dd_method(call),
        _ => false,
    }
}

fn extract_call_arguments(right: &Expr, args: &mut Vec<ExprOrSpread>) -> Expr {
    return match right {
        Expr::Call(call) => {
            call.args.iter().skip(1).for_each(|a| {
                args.push(a.clone());
            });
            *call.args[0].expr.clone()
        }
        _ => right.clone(),
    };
}
