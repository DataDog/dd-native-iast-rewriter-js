/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use swc::{atoms::JsWord, common::DUMMY_SP, ecmascript::ast::*};

pub struct TemplateTransform {}

impl TemplateTransform {
    pub fn get_binary_from_tpl(tpl: &Tpl) -> Expr {
        // at least 2 arguments (reversed): tail, expr, quasi (empty quasi `` is filtered)
        let arguments = get_reversed_arguments(tpl);

        let mut binary_expr = BinExpr {
            span: DUMMY_SP,
            op: BinaryOp::Add,
            left: Box::new(arguments[1].clone()),
            right: Box::new(arguments[0].clone()),
        };

        arguments.iter().skip(2).for_each(|arg| {
            binary_expr = BinExpr {
                span: DUMMY_SP,
                op: BinaryOp::Add,
                left: Box::new(arg.clone()),
                right: Box::new(Expr::Bin(binary_expr.clone())),
            }
        });

        Expr::Bin(binary_expr)
    }
}

fn get_reversed_arguments(tpl: &Tpl) -> Vec<Expr> {
    let mut arguments = Vec::new();
    let mut index = 0;
    let empty_quasi = JsWord::from("");
    for quasi in &tpl.quasis {
        let value = quasi.cooked.clone();
        if value.is_none() || value.unwrap() == empty_quasi {
            let expr = &*tpl.exprs[index];
            arguments.push(expr.clone());
            index += 1;
            continue;
        }

        // TODO: generate a Exp::Lit instead Expr::Tpl
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
            arguments.push(expr.clone());
        }

        index += 1;
    }
    arguments.reverse();
    arguments
}
