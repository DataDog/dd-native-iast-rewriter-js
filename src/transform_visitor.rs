use crate::transform_visitor::AssignOp::{AddAssign, Assign};
use crate::visitor_util::{
    dd_global_method_invocation, 
    get_plus_operator_based_on_num_of_args_for_span, 
    template_literal_operator, 
    two_items_plus_operator,
    NODE_GLOBAL, 
    DD_GLOBAL_NAMESPACE,
    DD_METHODS
};
use std::ops::Deref;
use swc::atoms::JsWord;
use swc::common::util::take::Take;
use swc::ecmascript::ast::*;
use swc_ecma_visit::{Visit, VisitMut, VisitMutWith};

pub struct TransformVisitor {
    pub counter: i32
}

impl TransformVisitor {}

impl Visit for TransformVisitor {}

impl VisitMut for TransformVisitor {
    fn visit_mut_assign_expr(&mut self, assign: &mut AssignExpr) {
        assign.visit_mut_children_with(self);
        if assign.op == AddAssign {
            assign.map_with_mut(|assign| to_dd_assign_expr(assign));
        }
    }

    fn visit_mut_expr(&mut self, expr: &mut Expr) {

        println!("original {:#?}", expr);
        let mut modified = false;
        expr.visit_mut_children_with(self);
        match expr {
            Expr::Bin(binary) => {
                if binary.op == BinaryOp::Add {
                    expr.map_with_mut(|bin| to_dd_binary_expr(bin));
                    modified = true;
                }
            }
            Expr::Tpl(tpl) => {
                if !tpl.exprs.is_empty() {
                    expr.map_with_mut(|tpl| to_dd_tpl_expr(tpl));
                }
            }
            _ => {}
        };

        if modified {
            println!("modified {:#?}", expr);
        }
    }
}


fn to_dd_assign_expr(assign: AssignExpr) -> AssignExpr {
    let span = assign.span;
    let op = assign.op;
    let left = assign.left;
    let right = assign.right;

    return match left {
        PatOrExpr::Pat(_) => AssignExpr {
            span,
            op,
            left,
            right,
        },
        PatOrExpr::Expr(left_expr) => {
            let left = *left_expr;
            let callee = dd_global_method_invocation(span, two_items_plus_operator);
            let args = vec![
                ExprOrSpread {
                    spread: None,
                    expr: Box::new(left.clone()),
                },
                ExprOrSpread {
                    spread: None,
                    expr: right,
                },
            ];
            let binary = Expr::Call(CallExpr {
                span,
                callee,
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

fn is_call_one_of_the_dd_methods_provided(call: &CallExpr, dd_methods: Vec<&str>) -> bool {
    let global_string = JsWord::from(NODE_GLOBAL);
    let dd_global_string = JsWord::from(DD_GLOBAL_NAMESPACE);
    let mut is_dd_method = false;
    let mut is_dd_global = false;
    let mut is_node_global = false;
    let mut coded_methods = Vec::new();
    let dd_methods_iter = dd_methods.iter();
    let callee: &MemberExpr;

    match &call.callee {
        Callee::Expr(call_calle) => {
            if let Expr::Member(c) = &**call_calle {
                callee = c;
            } else {
                return false;
            }
        }
        _ => {
            return false;
        }
    }

    for method in dd_methods_iter {
        coded_methods.push(JsWord::from(*method));
    }

    let x = callee.deref();
    if let MemberProp::Ident(ident) = &x.prop {
        if coded_methods.contains(&ident.sym) {
            is_dd_method = true;
        }
    }

    if let Expr::Member(member) = &x.obj.deref() {
        if let MemberProp::Ident(ident) = &member.prop {
            is_dd_global = ident.sym == dd_global_string;
        }

        if let Expr::Ident(ident) = &member.obj.deref() {
            is_node_global = ident.sym == global_string;
        }
    }

    return is_dd_method && is_dd_global && is_node_global;
}

fn is_dd_method(call: &CallExpr) -> bool {
    is_call_one_of_the_dd_methods_provided(
        call,
        DD_METHODS.to_vec()
    )
}

fn to_dd_binary_expr(expr: Expr) -> Expr {
    match expr {
        Expr::Bin(binary) => {
            let span = binary.span;
            let op = binary.op;
            let left = binary.left;
            let right = binary.right;
            let mut right_node_pushed: bool = false;

            if let Expr::Lit(_) = right.deref() {
                match left.deref() {
                    Expr::Lit(_) | Expr::Bin(_) => {
                        return Expr::Bin(BinExpr {
                            span,
                            op,
                            left,
                            right,
                        });
                    }
                    _ => {}
                }
            }

            let mut args: Vec<ExprOrSpread> = Vec::new();

            //Previous iteration was one of our methods. Clone args since we are going to modify them
            if let Expr::Call(call) = left.deref() {
                if is_dd_method(call) {
                    args = call.args.clone();
                }
            } else if let Expr::Call(call) = right.deref() {
                if is_dd_method(call) {
                    args = call.args.clone();
                    right_node_pushed = true;
                    args.insert(
                        0,
                        ExprOrSpread {
                            spread: None,
                            expr: left.clone(),
                        },
                    );
                }
            }

            //When this point reaches without args is because there was not a call to our methods
            if args.is_empty() {
                args.push(ExprOrSpread {
                    spread: None,
                    expr: left,
                });
            } else {
                //Handling parameters for our methods
                let last = args.last().unwrap().clone();

                if let Expr::Lit(_) = right.deref() {
                    match &*last.expr {
                        Expr::Lit(_) => {
                            //Last parameter passed to our method was literal and new one is literal. Then create binary with both
                            //Remove previous
                            args.pop();
                            //Add the new one
                            args.push(ExprOrSpread {
                                spread: None,
                                expr: Box::new(Expr::Bin(BinExpr {
                                    span,
                                    op,
                                    left: last.expr.clone(),
                                    right: right.clone(),
                                })),
                            });

                            right_node_pushed = true;
                        }
                        Expr::Bin(last_bin) => {
                            //Previous parameter was a binary of literals
                            if let Expr::Lit(_) = last_bin.left.deref() {
                                args.pop();

                                args.push(ExprOrSpread {
                                    spread: None,
                                    expr: Box::new(Expr::Bin(BinExpr {
                                        span,
                                        op,
                                        left: last_bin.left.clone(),
                                        right: Box::from(Expr::Bin(BinExpr {
                                            span,
                                            op,
                                            left: Box::new(*last_bin.right.clone()),
                                            right: right.clone(),
                                        })),
                                    })),
                                });

                                right_node_pushed = true;
                            }
                        }
                        _ => {}
                    }
                }
            }

            if !right_node_pushed {
                args.push(ExprOrSpread {
                    spread: None,
                    expr: right,
                });
            }

            Expr::Call(CallExpr {
                span,
                callee: get_plus_operator_based_on_num_of_args_for_span(args.len(), span),
                args,
                type_args: None,
            })
        }
        other => other,
    }
}

fn to_dd_tpl_expr(expr: Expr) -> Expr {
    let original_expr = expr.clone();
    match expr {
        Expr::Tpl(tpl) => {
            let span = tpl.span;
            let callee = dd_global_method_invocation(span, template_literal_operator);
            let mut args: Vec<ExprOrSpread> = Vec::new();
            let mut index = 0;
            let mut exprs = tpl.exprs.clone();
            let mut all_literals: bool = true;
            for quasi in tpl.quasis {
                let mut expr_args = Vec::new();
                expr_args.push(TplElement {
                    span: quasi.span,
                    tail: true,
                    cooked: quasi.cooked.clone(),
                    raw: quasi.raw,
                });
                let expr = Tpl {
                    span: tpl.span,
                    quasis: expr_args,
                    exprs: Vec::new(),
                };
                if quasi.cooked.clone().unwrap() != JsWord::from("") {
                    args.push(ExprOrSpread {
                        spread: None,
                        expr: Box::new(Expr::Tpl(expr)),
                    });
                }
                if !quasi.tail {
                    match *exprs[index] {
                        Expr::Lit(_) => {
                            //Nothing to do here
                        }
                        _ => {
                            all_literals = false;
                        }
                    }
                    args.push(ExprOrSpread {
                        spread: None,
                        expr: exprs[index].take(),
                    });
                    index += 1;
                }
            }
            if all_literals {
                return original_expr;
            }

            Expr::Call(CallExpr {
                span,
                callee,
                args,
                type_args: None,
            })
        }
        other => other,
    }
}


