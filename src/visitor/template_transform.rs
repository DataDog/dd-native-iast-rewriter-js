/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use swc::{atoms::JsWord, common::DUMMY_SP, ecmascript::ast::*};

pub struct TemplateTransform {}

impl TemplateTransform {
    pub fn get_binary_from_tpl(tpl: &Tpl) -> Expr {
        let arguments = get_reversed_arguments(tpl);

        let left: Expr;
        // with `${expression}` first quasi is filtered
        if arguments.len() == 1 {
            left = Expr::Lit(Lit::Str(Str {
                span: tpl.span,
                raw: None,
                value: JsWord::from(""),
            }))
        } else {
            left = arguments[1].clone()
        }

        let mut binary_expr = BinExpr {
            span: DUMMY_SP,
            op: BinaryOp::Add,
            left: Box::new(left),
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
            if !quasi.tail {
                let expr = &*tpl.exprs[index];
                arguments.push(expr.clone());
                index += 1;
            }
            continue;
        }

        let str = Expr::Lit(Lit::Str(Str {
            span: quasi.span,
            raw: None,
            value: quasi.cooked.clone().unwrap_or(empty_quasi.clone()),
        }));
        arguments.push(str);

        if !quasi.tail {
            let expr = &*tpl.exprs[index];
            arguments.push(expr.clone());
        }

        index += 1;
    }
    arguments.reverse();
    arguments
}
