/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use swc::{atoms::JsWord, common::util::take::Take, ecmascript::ast::*};

use super::{
    operation_transform_visitor::OperationTransformVisitor,
    visitor_util::get_dd_plus_operator_paren_expr,
};

pub struct TemplateTransform {}

impl TemplateTransform {
    pub fn to_dd_tpl_expr(expr: &Expr, opv: &mut OperationTransformVisitor) -> Expr {
        match expr {
            Expr::Tpl(tpl) => {
                let mut tpl_clone = tpl.clone();

                let mut assignations = Vec::new();
                let mut arguments = Vec::new();
                if prepare_replace_expressions_in_template(
                    &mut tpl_clone,
                    &mut assignations,
                    &mut arguments,
                    opv,
                ) {
                    return get_dd_plus_operator_paren_expr(
                        Expr::Tpl(tpl_clone),
                        &arguments,
                        &mut assignations,
                        tpl.span,
                    );
                }
            }
            _ => {}
        }
        expr.clone()
    }
}

fn prepare_replace_expressions_in_template(
    tpl: &mut Tpl,
    assignations: &mut Vec<Box<Expr>>,
    arguments: &mut Vec<Expr>,
    opv: &mut OperationTransformVisitor,
) -> bool {
    dbg!(&tpl);

    extract_arguments_in_template(tpl, arguments)
}

fn extract_arguments_in_template(tpl: &Tpl, args: &mut Vec<Expr>) -> bool {
    let mut index = 0;
    let mut exprs = tpl.exprs.clone();
    let mut all_literals: bool = true;
    for quasi in &tpl.quasis {
        let mut expr_args = Vec::new();
        expr_args.push(TplElement {
            span: quasi.span,
            tail: true,
            cooked: quasi.cooked.clone(),
            raw: quasi.raw.clone(),
        });
        let expr = Tpl {
            span: tpl.span,
            quasis: expr_args,
            exprs: Vec::new(),
        };
        if quasi.cooked.clone().unwrap() != JsWord::from("") {
            args.push(Expr::Tpl(expr));
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
            args.push(*exprs[index].take());
            index += 1;
        }
    }
    !all_literals
}
