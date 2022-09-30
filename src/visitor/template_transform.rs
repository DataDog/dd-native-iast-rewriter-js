/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use swc::{atoms::JsWord, common::Span, ecmascript::ast::*};

use super::{
    operation_transform_visitor::OperationTransformVisitor,
    visitor_util::{get_dd_plus_operator_paren_expr, is_typeof},
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
                    tpl.span,
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
    original_tpl: &mut Tpl,
    assignations: &mut Vec<Box<Expr>>,
    arguments: &mut Vec<Expr>,
    opv: &mut OperationTransformVisitor,
    span: Span,
) -> bool {
    let mut arguments_all = Vec::new();
    let replace_original_tpl =
        extract_arguments_in_template(original_tpl, assignations, &mut arguments_all, opv, span);
    if replace_original_tpl {
        // replace original template quasis and exprs with new expressions
        original_tpl.quasis.clear();
        original_tpl.exprs.clear();

        // we have to filter empty template arguments
        arguments_all.iter().for_each(|a| match a {
            Expr::Tpl(tpl) => {
                // here tpl always have a single quasi
                if tpl.quasis.len() > 0 && tpl.quasis[0].cooked.clone().unwrap() != JsWord::from("")
                {
                    arguments.push(a.clone());
                }
                original_tpl.quasis.append(&mut tpl.quasis.clone())
            }
            expr => {
                arguments.push(a.clone());
                original_tpl.exprs.push(Box::new(expr.clone()))
            }
        });
    }
    replace_original_tpl
}

fn extract_arguments_in_template(
    tpl: &mut Tpl,
    assignations: &mut Vec<Box<Expr>>,
    arguments: &mut Vec<Expr>,
    opv: &mut OperationTransformVisitor,
    span: Span,
) -> bool {
    let mut index = 0;
    let mut all_literals: bool = true;
    let mut pending_idents = Vec::new();
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
        arguments.push(Expr::Tpl(expr));

        if !quasi.tail {
            let expr = &*tpl.exprs[index];
            match expr {
                Expr::Lit(_) => {
                    arguments.push(expr.clone());
                }
                Expr::Unary(unary) => {
                    let is_typeof = is_typeof(&unary);
                    let ident = opv.get_ident_used_in_assignation_with_definitive(
                        expr.clone(),
                        assignations,
                        arguments,
                        span,
                        !is_typeof,
                    );

                    // replace operand with new ident
                    tpl.exprs[index] = Box::new(Expr::Ident(ident.clone()));

                    if is_typeof {
                        // the result of `typeof obj` is always a literal string so the ident is not replaced until we
                        // know that tpl contains some other not literal expression
                        pending_idents.push(ident);
                    } else {
                        all_literals = false;
                    }
                }
                _ => {
                    let ident = opv.get_ident_used_in_assignation(
                        expr.clone(),
                        assignations,
                        arguments,
                        span,
                    );

                    // replace operand with new ident
                    tpl.exprs[index] = Box::new(Expr::Ident(ident));

                    all_literals = false;
                }
            }
            index += 1;
        }
    }

    if !all_literals {
        pending_idents.iter().for_each(|id| {
            if !opv.idents.contains(id) {
                opv.idents.push(id.clone()); // another clone :(
            }
        });
    }

    !all_literals
}
