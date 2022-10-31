/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use crate::util::rnd_string;
use std::{env, sync::Once};
use swc::{atoms::JsWord, common::Span, ecmascript::ast::*};

const DATADOG_VAR_PREFIX: &str = "__datadog";
const DD_GLOBAL_NAMESPACE: &str = "_ddiast";
pub const DD_PLUS_OPERATOR: &str = "plusOperator";
pub const DD_LOCAL_VAR_NAME_HASH_ENV_NAME: &str = "DD_LOCAL_VAR_NAME_HASH";

static mut DD_LOCAL_VAR_NAME_HASH: String = String::new();
static DD_LOCAL_VAR_NAME_HASH_INIT: Once = Once::new();
pub fn get_dd_local_var_name_hash() -> String {
    unsafe {
        DD_LOCAL_VAR_NAME_HASH_INIT.call_once(|| match env::var(DD_LOCAL_VAR_NAME_HASH_ENV_NAME) {
            Ok(val) => {
                DD_LOCAL_VAR_NAME_HASH = val;
            }
            Err(_) => {
                DD_LOCAL_VAR_NAME_HASH = rnd_string(6);
            }
        });
        DD_LOCAL_VAR_NAME_HASH.clone()
    }
}

pub fn get_dd_local_variable_name(n: usize) -> String {
    format!("{}{}", get_dd_local_variable_prefix(), n)
}

pub fn get_dd_local_variable_prefix() -> String {
    format!("{}_{}_", DATADOG_VAR_PREFIX, get_dd_local_var_name_hash())
}

pub fn dd_global_method_invocation(method_name: &str, span: &Span) -> Callee {
    Callee::Expr(Box::new(Expr::Member(MemberExpr {
        span: *span,
        prop: MemberProp::Ident(Ident {
            span: *span,
            sym: JsWord::from(method_name),
            optional: false,
        }),
        obj: Box::new(Expr::Ident(Ident {
            span: *span,
            sym: JsWord::from(DD_GLOBAL_NAMESPACE),
            optional: false,
        })),
    })))
}

pub fn get_dd_call_expr(expr: &Expr, arguments: &[Expr], method_name: &str, span: &Span) -> Expr {
    let mut args: Vec<ExprOrSpread> = vec![ExprOrSpread {
        expr: Box::new(expr.clone()),
        spread: None,
    }];

    args.append(
        &mut arguments
            .iter()
            .map(|expr| ExprOrSpread {
                expr: Box::new(expr.clone()),
                spread: None,
            })
            .collect::<Vec<_>>(),
    );

    Expr::Call(CallExpr {
        span: *span,
        callee: dd_global_method_invocation(method_name, span),
        args,
        type_args: None,
    })
}

pub fn get_dd_plus_operator_paren_expr(
    expr: &Expr,
    arguments: &[Expr],
    assignations: &mut Vec<Expr>,
    span: &Span,
) -> Expr {
    get_dd_paren_expr(expr, arguments, assignations, DD_PLUS_OPERATOR, span)
}

pub fn get_dd_paren_expr(
    expr: &Expr,
    arguments: &[Expr],
    assignations: &mut Vec<Expr>,
    method_name: &str,
    span: &Span,
) -> Expr {
    let call = get_dd_call_expr(expr, arguments, method_name, span);

    // if there are 0 assign expressions we can return just call expression without parentheses
    // else wrap them all with a sequence of comma separated expressions inside parentheses
    if assignations.is_empty() {
        call
    } else {
        assignations.push(call);
        return Expr::Paren(ParenExpr {
            span: *span,
            expr: Box::new(Expr::Seq(SeqExpr {
                span: *span,
                exprs: assignations
                    .iter()
                    .map(|assignation| Box::new(assignation.clone()))
                    .collect::<Vec<Box<Expr>>>(),
            })),
        });
    }
}
