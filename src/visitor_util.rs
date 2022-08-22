use random_string::generate;
use std::env;
use std::sync::Once;
use swc::atoms::JsWord;
use swc::common::Span;
use swc::ecmascript::ast::*;

pub const NODE_GLOBAL: &str = "global";
pub const DD_GLOBAL_NAMESPACE: &str = "_ddiast";
const DD_TWO_ITEMS_PLUS_OPERATOR: &str = "twoItemsPlusOperator";
const DD_THREE_ITEMS_PLUS_OPERATOR: &str = "threeItemsPlusOperator";
const DD_FOUR_ITEMS_PLUS_OPERATOR: &str = "fourItemsPlusOperator";
const DD_FIVE_ITEMS_PLUS_OPERATOR: &str = "fiveItemsPlusOperator";
const DD_ANY_PLUS_OPERATOR: &str = "anyPlusOperator";

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

pub const DD_METHODS: &[&str] = &[
    DD_TWO_ITEMS_PLUS_OPERATOR,
    DD_THREE_ITEMS_PLUS_OPERATOR,
    DD_FOUR_ITEMS_PLUS_OPERATOR,
    DD_FIVE_ITEMS_PLUS_OPERATOR,
    DD_ANY_PLUS_OPERATOR,
];

pub fn get_plus_operator_based_on_num_of_args_for_span(arguments_len: usize, span: Span) -> Callee {
    match arguments_len {
        2 => return dd_global_method_invocation(span, two_items_plus_operator),
        3 => return dd_global_method_invocation(span, three_items_plus_operator),
        4 => return dd_global_method_invocation(span, four_items_plus_operator),
        5 => return dd_global_method_invocation(span, five_items_plus_operator),
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

pub fn two_items_plus_operator(span: Span) -> MemberProp {
    MemberProp::Ident(Ident {
        span,
        sym: JsWord::from(DD_TWO_ITEMS_PLUS_OPERATOR),
        optional: false,
    })
}

fn three_items_plus_operator(span: Span) -> MemberProp {
    MemberProp::Ident(Ident {
        span,
        sym: JsWord::from(DD_THREE_ITEMS_PLUS_OPERATOR),
        optional: false,
    })
}

fn four_items_plus_operator(span: Span) -> MemberProp {
    MemberProp::Ident(Ident {
        span,
        sym: JsWord::from(DD_FOUR_ITEMS_PLUS_OPERATOR),
        optional: false,
    })
}

fn five_items_plus_operator(span: Span) -> MemberProp {
    MemberProp::Ident(Ident {
        span,
        sym: JsWord::from(DD_FIVE_ITEMS_PLUS_OPERATOR),
        optional: false,
    })
}

fn any_items_plus_operator(span: Span) -> MemberProp {
    MemberProp::Ident(Ident {
        span,
        sym: JsWord::from(DD_ANY_PLUS_OPERATOR),
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
