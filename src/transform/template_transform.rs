/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use swc::atoms::JsWord;
use swc_common::{SyntaxContext, DUMMY_SP};
use swc_ecma_ast::*;

use super::function_prototype_transform::FunctionPrototypeTransform;

pub struct TemplateTransform {}

impl TemplateTransform {
    pub fn get_concat_from_tpl(tpl: &Tpl) -> CallExpr {
        let args = get_arguments(tpl);

        CallExpr {
            span: DUMMY_SP,
            args,
            ctxt: SyntaxContext::empty(),
            type_args: None,
            callee: Callee::Expr(Box::new(Expr::Member(
                FunctionPrototypeTransform::get_member_expr_from_path(
                    "String.prototype.concat.call",
                ),
            ))),
        }
    }
}

fn get_arguments(tpl: &Tpl) -> Vec<ExprOrSpread> {
    let mut arguments = Vec::new();
    let mut index = 0;
    let empty_quasi = JsWord::from("");
    for quasi in &tpl.quasis {
        let value = quasi.cooked.clone();
        if value.is_none() || value.unwrap() == empty_quasi {
            if !quasi.tail {
                let expr = &*tpl.exprs[index];
                arguments.push(ExprOrSpread::from(expr.clone()));
                index += 1;
            }
            continue;
        }

        let str = Expr::Lit(Lit::Str(Str {
            span: quasi.span,
            raw: None,
            value: quasi.cooked.clone().unwrap_or_else(|| empty_quasi.clone()),
        }));
        arguments.push(ExprOrSpread::from(str));

        if !quasi.tail {
            let expr = &*tpl.exprs[index];
            arguments.push(ExprOrSpread::from(expr.clone()));
        }

        index += 1;
    }
    arguments
}
