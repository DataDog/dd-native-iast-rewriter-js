use std::ops::DerefMut;
use swc::{
    atoms::JsWord,
    common::{util::take::Take, Span},
    ecmascript::ast::*,
};
use swc_ecma_visit::{Visit, VisitMut, VisitMutWith};

use crate::{
    assign_transform_visitor::AssignTransformVisitor,
    visitor_util::{get_dd_local_variable_name, get_plus_operator_based_on_num_of_args_for_span},
};

pub struct OperationTransformVisitor {
    pub assign_visitor: AssignTransformVisitor,
    pub counter: usize,
}

impl Visit for OperationTransformVisitor {}

impl VisitMut for OperationTransformVisitor {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        //println!("expr {:#?}", expr);
        match expr {
            Expr::Bin(binary) => {
                if binary.op == BinaryOp::Add {
                    expr.map_with_mut(|bin| to_dd_binary_expr(&bin, self));
                    return;
                } else {
                    expr.visit_mut_children_with(self);
                }
            }
            Expr::Assign(assign) => {
                assign.visit_mut_children_with(self);
                self.assign_visitor.visit_mut_assign_expr(assign);
            }
            _ => {
                expr.visit_mut_children_with(self);
            }
        }
    }

    fn visit_mut_if_stmt(&mut self, if_stmt: &mut IfStmt) {
        if_stmt.test.visit_mut_children_with(self);
    }
    fn visit_mut_block_stmt(&mut self, _expr: &mut BlockStmt) {
        //println!("expr block {:#?}", _expr);
    }
}

fn to_dd_binary_expr(
    expr: &Expr,
    operation_transform_visitor: &mut OperationTransformVisitor,
) -> Expr {
    match expr {
        Expr::Bin(binary) => {
            let mut binary_clone = binary.clone();
            let span = binary.span;

            let mut assign_expressions = Vec::new();
            let mut arguments = Vec::new();
            replace_expressions_in_binary(
                &mut binary_clone,
                &mut assign_expressions,
                &mut arguments,
                operation_transform_visitor,
            );

            operation_transform_visitor.counter = assign_expressions.len();

            // if all arguments are literals we can skip expression replacement
            if must_replace_binary_expression(&arguments) {
                let plus_operator_call = get_dd_call_plus_operator_expr(binary_clone, &arguments);

                // if there are 0 assign expressions we can return just call expression without parentheses
                // else wrap them all with a sequence of comma separated expressions inside parentheses
                if assign_expressions.len() == 0 {
                    return plus_operator_call;
                } else {
                    assign_expressions.push(Box::new(plus_operator_call));
                    return Expr::Paren(ParenExpr {
                        span,
                        expr: Box::new(Expr::Seq(SeqExpr {
                            span,
                            exprs: assign_expressions,
                        })),
                    });
                }
            }
        }
        _ => {}
    }
    expr.clone()
}

fn must_replace_binary_expression(argument_exprs: &Vec<Expr>) -> bool {
    // by now only literals a filtered but may be other cases.
    argument_exprs.iter().any(|arg| match arg {
        Expr::Lit(_) => false,
        _ => true,
    })
}

fn create_assign_expression(index: usize, expr: Expr, span: Span) -> (Expr, Expr) {
    let id = Ident {
        span,
        sym: JsWord::from(get_dd_local_variable_name(index)),
        optional: false,
    };
    (
        Expr::Assign(AssignExpr {
            span,
            left: PatOrExpr::Pat(Box::new(Pat::Ident(BindingIdent {
                id: id.clone(),
                type_ann: None,
            }))),
            right: Box::new(expr),
            op: AssignOp::Assign,
        }),
        Expr::Ident(id),
    )
}

fn replace_expressions_in_binary(
    binary: &mut BinExpr,
    assign_exprs: &mut Vec<Box<Expr>>,
    argument_exprs: &mut Vec<Expr>,
    operation_transform_visitor: &mut OperationTransformVisitor,
) {
    let left = binary.left.deref_mut();
    replace_expressions_in_binary_operand(
        left,
        assign_exprs,
        argument_exprs,
        binary.span,
        operation_transform_visitor,
    );

    let right = binary.right.deref_mut();
    replace_expressions_in_binary_operand(
        right,
        assign_exprs,
        argument_exprs,
        binary.span,
        operation_transform_visitor,
    );
}

fn replace_expressions_in_binary_operand(
    operand: &mut Expr,
    assign_exprs: &mut Vec<Box<Expr>>,
    argument_exprs: &mut Vec<Expr>,
    span: Span,
    operation_transform_visitor: &mut OperationTransformVisitor,
) {
    match operand {
        Expr::Bin(binary) => {
            if binary.op == BinaryOp::Add {
                replace_expressions_in_binary(
                    binary,
                    assign_exprs,
                    argument_exprs,
                    operation_transform_visitor,
                );
            } else {
                argument_exprs.push(operand.clone())
            }
        }
        Expr::Call(_) | Expr::Paren(_) => {
            // visit_mut_children_with maybe only needed by Paren but...
            operand.visit_mut_children_with(operation_transform_visitor);

            let index = assign_exprs.len();
            let (assign, id) = create_assign_expression(index, operand.clone(), span);
            assign_exprs.push(Box::new(assign));
            operand.map_with_mut(|_| get_ident(span, index));
            argument_exprs.push(id);
        }
        _ => {
            //println!("operand {:#?}", operand);
            argument_exprs.push(operand.clone())
        }
    }
}

fn get_ident(span: Span, index: usize) -> Expr {
    return Expr::Ident(Ident {
        span,
        sym: JsWord::from(get_dd_local_variable_name(index)),
        optional: false,
    });
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
