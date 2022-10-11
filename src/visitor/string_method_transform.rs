/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use swc::common::{util::take::Take, Span};
use swc_ecma_visit::swc_ecma_ast::{CallExpr, Callee, Expr, Ident, MemberExpr, MemberProp};

use crate::visitor::function_prototype_transform::FunctionPrototypeTransform;

use super::{
    operation_transform_visitor::OperationTransformVisitor, visitor_util::get_dd_paren_expr,
};

pub const STRING_CLASS_NAME: &str = "String";

// TODO: make vector static with Once?
fn get_methods() -> Vec<String> {
    vec!["substring".to_string()]
}

pub struct StringMethodTransform {}

impl StringMethodTransform {
    pub fn to_dd_string_expr(
        call: &mut CallExpr,
        opv: &mut OperationTransformVisitor,
    ) -> Option<Expr> {
        let callee = call.callee.clone();
        match callee {
            Callee::Expr(expr) => match *expr {
                Expr::Member(member) => match (*member.obj, member.prop) {
                    // replace ident and call members but exclude literal "".substring() calls
                    (Expr::Ident(obj), MemberProp::Ident(ident)) => {
                        replace_call_expr_if_match(Expr::Ident(obj), ident, call, opv)
                    }
                    (Expr::Call(callee_call), MemberProp::Ident(ident)) => {
                        replace_call_expr_if_match(Expr::Call(callee_call), ident, call, opv)
                    }

                    // may be something like String.prototype.substring.call
                    (Expr::Member(member_obj), MemberProp::Ident(ident)) => {
                        let prototype_call_option =
                            FunctionPrototypeTransform::get_ident_from_call_or_apply(
                                &call,
                                &member_obj,
                                &ident,
                                STRING_CLASS_NAME,
                            );
                        if prototype_call_option.is_some() {
                            let mut prototype_call = prototype_call_option.unwrap();
                            replace_call_expr_if_match(
                                prototype_call.2,
                                prototype_call.0,
                                &mut prototype_call.1,
                                opv,
                            )
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }
    }
}

fn replace_call_expr_if_match(
    expr: Expr,
    ident: Ident,
    call: &mut CallExpr,
    opv: &mut OperationTransformVisitor,
) -> Option<Expr> {
    let method_name = ident.sym.to_string();
    if get_methods().contains(&method_name) {
        let mut assignations = Vec::new();
        let mut arguments = Vec::new();
        let span = call.span;

        // always generate a new ident and replace the original callee with it:
        // a.substring() -> __datadog_token_$i.substring()
        // a().substring() -> __datadog_token_$i.substring()
        let mut call_replacement = call.clone();
        let ident_replacement =
            opv.get_ident_used_in_assignation(expr, &mut assignations, &mut arguments, span);
        call_replacement.callee = Callee::Expr(Box::new(Expr::Member(MemberExpr {
            span,
            obj: Box::new(Expr::Ident(ident_replacement)),
            prop: MemberProp::Ident(ident),
        })));

        call_replacement.args.iter_mut().for_each(|expr_or_span| {
            replace_expressions_in_operand(
                &mut *expr_or_span.expr,
                &mut assignations,
                &mut arguments,
                span,
                opv,
            );
        });

        return Some(get_dd_paren_expr(
            Expr::Call(call_replacement),
            &arguments,
            &mut assignations,
            &method_name,
            span,
        ));
    }
    None
}

fn replace_expressions_in_operand(
    operand: &mut Expr,
    assignations: &mut Vec<Box<Expr>>,
    arguments: &mut Vec<Expr>,
    span: Span,
    opv: &mut OperationTransformVisitor,
) {
    match operand {
        Expr::Lit(_) => arguments.push(operand.clone()),

        _ => operand.map_with_mut(|op| {
            Expr::Ident(opv.get_ident_used_in_assignation(op, assignations, arguments, span))
        }),
    }
}
