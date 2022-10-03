/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use crate::util::rnd_string;
use std::{env, sync::Once};
use swc::{
    atoms::JsWord,
    common::{Span, DUMMY_SP},
    ecmascript::ast::*,
};

pub const DD_GLOBAL_NAMESPACE: &str = "_ddiast";
const DD_PLUS_OPERATOR: &str = "plusOperator";
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
    format!("__datadog_{}_", get_dd_local_var_name_hash())
}

pub fn get_plus_operator_based_on_num_of_args_for_span(arguments_len: usize, span: Span) -> Callee {
    match arguments_len {
        _other => dd_global_method_invocation(span, any_items_plus_operator),
    }
}

pub fn dd_global_method_invocation<F>(span: Span, method: F) -> Callee
where
    F: FnOnce(Span) -> MemberProp,
{
    Callee::Expr(Box::new(Expr::Member(MemberExpr {
        span,
        prop: method(span),
        obj: Box::new(Expr::Ident(Ident {
            span,
            sym: JsWord::from(DD_GLOBAL_NAMESPACE),
            optional: false,
        })),
    })))
}

pub fn any_items_plus_operator(span: Span) -> MemberProp {
    MemberProp::Ident(Ident {
        span,
        sym: JsWord::from(DD_PLUS_OPERATOR),
        optional: false,
    })
}

pub fn get_dd_call_plus_operator_expr(expr: Expr, arguments: &Vec<Expr>, span: Span) -> Expr {
    let mut args: Vec<ExprOrSpread> = Vec::new();

    args.push(ExprOrSpread {
        expr: Box::new(expr),
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

pub fn get_dd_plus_operator_paren_expr(
    expr: Expr,
    arguments: &Vec<Expr>,
    assignations: &mut Vec<Box<Expr>>,
    span: Span,
) -> Expr {
    let plus_operator_call = get_dd_call_plus_operator_expr(expr, &arguments, span);

    // if there are 0 assign expressions we can return just call expression without parentheses
    // else wrap them all with a sequence of comma separated expressions inside parentheses
    if assignations.len() == 0 {
        return plus_operator_call;
    } else {
        assignations.push(Box::new(plus_operator_call));
        return Expr::Paren(ParenExpr {
            span,
            expr: Box::new(Expr::Seq(SeqExpr {
                span,
                exprs: assignations.clone(),
            })),
        });
    }
}

pub fn create_assign_expression(index: usize, expr: Expr, span: Span) -> (AssignExpr, Ident) {
    let id = Ident {
        span: DUMMY_SP,
        sym: JsWord::from(get_dd_local_variable_name(index)),
        optional: false,
    };
    (
        AssignExpr {
            span,
            left: PatOrExpr::Pat(Box::new(Pat::Ident(BindingIdent {
                id: id.clone(),
                type_ann: None,
            }))),
            right: Box::new(expr),
            op: AssignOp::Assign,
        },
        id,
    )
}
