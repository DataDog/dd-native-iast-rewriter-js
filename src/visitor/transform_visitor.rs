use crate::visitor::visitor_util::{dd_global_method_invocation, template_literal_operator};

use swc::{atoms::JsWord, common::util::take::Take, ecmascript::ast::*};
use swc_ecma_visit::{Visit, VisitMut, VisitMutWith};

pub struct TransformVisitor {
    pub counter: i32,
}

impl TransformVisitor {}

impl Visit for TransformVisitor {}

impl VisitMut for TransformVisitor {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        println!("original {:#?}", expr);
        let mut modified = false;
        expr.visit_mut_children_with(self);
        match expr {
            Expr::Tpl(tpl) => {
                if !tpl.exprs.is_empty() {
                    expr.map_with_mut(|tpl| to_dd_tpl_expr(tpl));
                    modified = true;
                }
            }
            _ => {}
        };

        if modified {
            println!("modified {:#?}", expr);
        }
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
