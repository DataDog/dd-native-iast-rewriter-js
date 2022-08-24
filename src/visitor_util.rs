use random_string::generate;
use std::{env, ops::Deref, sync::Once};
use swc::{atoms::JsWord, common::Span, ecmascript::ast::*};

pub const NODE_GLOBAL: &str = "global";
pub const DD_GLOBAL_NAMESPACE: &str = "_ddiast";
const DD_PLUS_OPERATOR: &str = "plusOperator";

static mut DD_LOCAL_VAR_NAME_HASH: String = String::new();
static DD_LOCAL_VAR_NAME_HASH_INIT: Once = Once::new();
pub fn get_dd_local_var_name_hash() -> String {
    unsafe {
        DD_LOCAL_VAR_NAME_HASH_INIT.call_once(|| match env::var("DD_LOCAL_VAR_NAME_HASH") {
            Ok(val) => {
                DD_LOCAL_VAR_NAME_HASH = val;
            }
            Err(_) => {
                DD_LOCAL_VAR_NAME_HASH = generate(6, "abcdefghijklmnopqrstuvwxyz");
            }
        });
        DD_LOCAL_VAR_NAME_HASH.clone()
    }
}

pub fn get_dd_local_variable_name(n: usize) -> String {
    format!("__datadog_{}_{}", get_dd_local_var_name_hash(), n)
}

pub const DD_METHODS: &[&str] = &[DD_PLUS_OPERATOR];

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
        obj: Box::new(Expr::Member(MemberExpr {
            span,
            prop: MemberProp::Ident(Ident {
                span,
                sym: JsWord::from(DD_GLOBAL_NAMESPACE),
                optional: false,
            }),
            obj: Box::new(Expr::Ident(Ident {
                span,
                sym: JsWord::from(NODE_GLOBAL),
                optional: false,
            })),
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

pub fn template_literal_operator(span: Span) -> MemberProp {
    MemberProp::Ident(Ident {
        span,
        sym: JsWord::from("templateLiteralOperator"),
        optional: false,
    })
}

pub fn is_dd_method(call: &CallExpr) -> bool {
    is_call_one_of_the_dd_methods_provided(call, DD_METHODS.to_vec())
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
