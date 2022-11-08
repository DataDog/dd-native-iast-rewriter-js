/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use swc::atoms::JsWord;
use swc_ecma_visit::swc_ecma_ast::{
    CallExpr, Callee, Expr, ExprOrSpread, Ident, MemberExpr, MemberProp,
};

use crate::{
    transform::{
        function_prototype_transform::FunctionPrototypeTransform,
        operand_handler::{DefaultOperandHandler, OperandHandler},
    },
    visitor::{
        csi_methods::CsiMethods, ident_provider::IdentProvider, transform_status::TransformStatus,
    },
};

use crate::visitor::visitor_util::get_dd_paren_expr;

use super::operand_handler::IdentMode;

pub struct CallExprTransform {}

impl CallExprTransform {
    pub fn to_dd_call_expr(
        call: &mut CallExpr,
        csi_methods: &CsiMethods,
        ident_provider: &mut dyn IdentProvider,
    ) -> Option<Expr> {
        let callee = call.callee.clone();
        match callee {
            Callee::Expr(expr) => match *expr {
                Expr::Member(member) => match (*member.obj, &member.prop) {
                    // replace ident and call members but exclude literal "".substring() calls
                    (Expr::Ident(obj), MemberProp::Ident(ident)) => {
                        replace_call_expr_if_csi_method(
                            &Expr::Ident(obj),
                            ident,
                            call,
                            csi_methods,
                            ident_provider,
                        )
                    }
                    (Expr::Call(callee_call), MemberProp::Ident(ident)) => {
                        replace_call_expr_if_csi_method(
                            &Expr::Call(callee_call),
                            ident,
                            call,
                            csi_methods,
                            ident_provider,
                        )
                    }

                    (Expr::Paren(paren), MemberProp::Ident(ident)) => {
                        replace_call_expr_if_csi_method(
                            &Expr::Paren(paren),
                            ident,
                            call,
                            csi_methods,
                            ident_provider,
                        )
                    }

                    (Expr::Member(member_obj), MemberProp::Ident(ident)) => {
                        // may be something like String.prototype.substring.call
                        if FunctionPrototypeTransform::is_call_or_apply(ident) {
                            replace_prototype_call_or_apply(
                                call,
                                &member_obj,
                                ident,
                                csi_methods,
                                ident_provider,
                            )

                        // or a.b.substring() but not String.prototype.substring()
                        } else {
                            if FunctionPrototypeTransform::member_prop_is_prototype(&member_obj) {
                                return None;
                            }

                            replace_call_expr_if_csi_method(
                                &Expr::Member(member_obj),
                                ident,
                                call,
                                csi_methods,
                                ident_provider,
                            )
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

fn replace_prototype_call_or_apply(
    call: &CallExpr,
    member: &MemberExpr,
    ident: &Ident,
    csi_methods: &CsiMethods,
    ident_provider: &mut dyn IdentProvider,
) -> Option<Expr> {
    let prototype_call_option =
        FunctionPrototypeTransform::get_expression_parts_from_call_or_apply(call, member, ident);

    match prototype_call_option {
        Some(mut prototype_call) => replace_call_expr_if_csi_method(
            &prototype_call.0,
            &prototype_call.1,
            &mut prototype_call.2,
            csi_methods,
            ident_provider,
        ),
        _ => None,
    }
}

fn replace_call_expr_if_csi_method(
    expr: &Expr,
    ident: &Ident,
    call: &mut CallExpr,
    csi_methods: &CsiMethods,
    ident_provider: &mut dyn IdentProvider,
) -> Option<Expr> {
    let method_name = &ident.sym.to_string();
    if let Some(csi_method) = csi_methods.get(method_name) {
        let mut assignations = Vec::new();
        let mut arguments = Vec::new();
        let span = call.span;

        // replace original call expression with an parent expression splitting every component and finally invoking .call
        // a.substring() ->  __datadog_token_$i = a, __datadog_token_$i2 = __datadog_token_$i.substring, __datadog_token_$i2.call(__datadog_token_$i, __datadog_token_$i2)

        let mut call_replacement = call.clone();

        // __datadog_token_$i = a
        let ident_replacement =
            ident_provider.get_temporal_ident_used_in_assignation(expr, &mut assignations, &span);

        // __datadog_token_$i.substring
        let member_expr = Expr::Member(MemberExpr {
            span,
            obj: Box::new(Expr::Ident(ident_replacement.clone())),
            prop: MemberProp::Ident(ident.clone()),
        });

        // __datadog_token_$i2 = __datadog_token_$i.substring
        let ident_callee = ident_provider.get_ident_used_in_assignation(
            &member_expr,
            &mut assignations,
            &mut arguments,
            &span,
        );

        arguments.push(Expr::Ident(ident_replacement.clone()));

        // change callee to __datadog_token_$i2.call
        call_replacement.callee = Callee::Expr(Box::new(Expr::Member(MemberExpr {
            span,
            obj: Box::new(Expr::Ident(ident_callee)),
            prop: MemberProp::Ident(Ident {
                span,
                sym: JsWord::from("call"),
                optional: false,
            }),
        })));
        call_replacement.args.iter_mut().for_each(|expr_or_spread| {
            DefaultOperandHandler::replace_expressions_in_operand(
                &mut *expr_or_spread.expr,
                IdentMode::Replace,
                &mut assignations,
                &mut arguments,
                &span,
                ident_provider,
            )
        });
        // insert .call(this) argument
        call_replacement.args.insert(
            0,
            ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Ident(ident_replacement)),
            },
        );

        ident_provider.set_status(TransformStatus::modified());

        return Some(get_dd_paren_expr(
            &Expr::Call(call_replacement),
            &arguments,
            &mut assignations,
            csi_method.get_dst().as_str(),
            &span,
        ));
    }
    None
}
