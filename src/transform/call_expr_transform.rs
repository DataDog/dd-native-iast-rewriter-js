/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use swc::atoms::JsWord;
use swc_common::SyntaxContext;
use swc_ecma_ast::*;

use crate::{
    transform::{
        function_prototype_transform::FunctionPrototypeTransform,
        operand_handler::{DefaultOperandHandler, OperandHandler},
    },
    visitor::{
        csi_methods::CsiMethods,
        ident_provider::{IdentKind, IdentProvider},
    },
};

use crate::visitor::visitor_util::get_dd_paren_expr;

use super::{
    operand_handler::{ExpandArrays, IdentMode},
    transform_status::TransformResult,
};

pub struct ResultExpr {
    pub expr: Expr,
    pub tag: String,
}

pub struct CallExprTransform {}

impl CallExprTransform {
    pub fn to_dd_call_expr(
        call: &mut CallExpr,
        csi_methods: &CsiMethods,
        ident_provider: &mut dyn IdentProvider,
    ) -> TransformResult<Expr> {
        let callee = call.callee.clone();
        let optional_result_expr = match callee {
            Callee::Expr(expr) => match *expr {
                Expr::Member(member) => match (*member.obj, &member.prop) {
                    // replace ident and call members, exclude literal "".substring() calls but do not exclude "literal".concat(a, b, c)
                    (Expr::Lit(literal), MemberProp::Ident(ident)) => {
                        if csi_methods.method_allows_literal_callers(&ident.sym) {
                            replace_call_expr_if_csi_method(
                                &Expr::Lit(literal),
                                ident,
                                call,
                                csi_methods,
                                ident_provider,
                            )
                        } else {
                            None
                        }
                    }

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

                    (Expr::Array(paren), MemberProp::Ident(ident)) => {
                        replace_call_expr_if_csi_method(
                            &Expr::Array(paren),
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
                        } else if !FunctionPrototypeTransform::member_prop_is_prototype(&member_obj)
                        {
                            replace_call_expr_if_csi_method(
                                &Expr::Member(member_obj),
                                ident,
                                call,
                                csi_methods,
                                ident_provider,
                            )
                        } else {
                            None
                        }
                    }
                    _ => None,
                },

                Expr::Ident(obj) => replace_call_expr_if_csi_method_without_callee(
                    &obj,
                    call,
                    csi_methods,
                    ident_provider,
                ),
                _ => None,
            },
            _ => None,
        };

        if let Some(result_expr) = optional_result_expr {
            TransformResult::modified_with_tag(result_expr.expr, result_expr.tag)
        } else {
            TransformResult::not_modified()
        }
    }
}

fn replace_prototype_call_or_apply(
    call: &CallExpr,
    member: &MemberExpr,
    ident_name: &IdentName,
    csi_methods: &CsiMethods,
    ident_provider: &mut dyn IdentProvider,
) -> Option<ResultExpr> {
    let prototype_call_option = FunctionPrototypeTransform::get_expression_parts_from_call_or_apply(
        call,
        member,
        ident_name,
        csi_methods,
    );

    match prototype_call_option {
        Some(mut prototype_call) => replace_call_expr_or_spread_if_csi_method_with_member(
            &prototype_call.0,
            &prototype_call.1.into(),
            &mut prototype_call.2,
            csi_methods,
            member,
            &ident_name.sym,
            ident_provider,
        ),
        _ => None,
    }
}

fn replace_call_expr_if_csi_method(
    expr: &Expr,
    ident_name: &IdentName,
    call: &mut CallExpr,
    csi_methods: &CsiMethods,
    ident_provider: &mut dyn IdentProvider,
) -> Option<ResultExpr> {
    replace_call_expr_if_csi_method_with_member(
        expr,
        ident_name,
        call,
        csi_methods,
        None,
        None,
        ident_provider,
    )
}

fn replace_call_expr_if_csi_method_without_callee(
    ident: &Ident,
    call: &mut CallExpr,
    csi_methods: &CsiMethods,
    ident_provider: &mut dyn IdentProvider,
) -> Option<ResultExpr> {
    let method_name = &ident.sym.to_string();

    if let Some(csi_method) = csi_methods.get(method_name) {
        if csi_method.allowed_without_callee {
            let mut assignations = Vec::new();
            let mut arguments = Vec::new();
            let span = call.span;

            // let __datadog_test_0;
            // (__datadog_test_0 = arg0, _ddiast.aloneMethod
            // (aloneMethod(__datadog_test_0), aloneMethod, undefined, __datadog_test_0));
            arguments.push(ExprOrSpread::from(Expr::Ident(ident.clone())));
            let global = Ident {
                span,
                sym: JsWord::from("undefined"),
                optional: false,
                ctxt: SyntaxContext::empty(),
            };
            arguments.push(ExprOrSpread::from(Expr::Ident(global)));

            let call_replacement = replace_call_callee_and_args(
                call,
                None,
                &mut assignations,
                &mut arguments,
                None,
                ident_provider,
            );

            return Some(ResultExpr {
                tag: method_name.clone(),
                expr: get_dd_paren_expr(
                    &Expr::Call(call_replacement),
                    &arguments,
                    &mut assignations,
                    csi_method.dst.as_str(),
                    &span,
                ),
            });
        }
    }
    None
}

fn replace_call_expr_if_csi_method_with_member(
    expr: &Expr,
    ident_name: &IdentName,
    call: &mut CallExpr,
    csi_methods: &CsiMethods,
    member_expr_opt: Option<&MemberExpr>,
    call_or_apply: Option<&str>,
    ident_provider: &mut dyn IdentProvider,
) -> Option<ResultExpr> {
    let method_name = &ident_name.sym.to_string();

    if let Some(csi_method) = csi_methods.get(method_name) {
        let mut assignations = Vec::new();
        let mut arguments = Vec::new();
        let span = call.span;

        // replace original call expression with a parent expression splitting every component and finally invoking .call
        //  a) a.substring() -> __datadog_token_$i = a, __datadog_token_$i2 = __datadog_token_$i.substring, __datadog_token_$i2.call(__datadog_token_$i, __datadog_token_$i2)
        //  b) String.prototype.substring.[call|apply](a) -> __datadog_token_$i = a, __datadog_token_$i2 = String.prototype.substring, __datadog_token_$i2.call(__datadog_token_$i, __datadog_token_$i2)

        // __datadog_token_$i = a
        let ident_replacement_option = ident_provider.get_temporal_ident_used_in_assignation(
            expr,
            &mut assignations,
            &span,
            IdentKind::Expr,
        );

        let ident_replacement = ident_replacement_option.map_or_else(|| expr.clone(), Expr::Ident);

        let ident_callee = match member_expr_opt {
            Some(member_expr) => {
                // __datadog_token_$i2 = member
                ident_provider.get_ident_used_in_assignation(
                    &Expr::Member(member_expr.clone()),
                    &mut assignations,
                    &mut arguments,
                    &span,
                    IdentKind::Expr,
                )
            }
            None => {
                // __datadog_token_$i.substring
                let member_expr = MemberExpr {
                    span,
                    obj: Box::new(ident_replacement.clone()),
                    prop: MemberProp::Ident(ident_name.clone()),
                };

                // __datadog_token_$i2 = __datadog_token_$i.substring
                ident_provider.get_ident_used_in_assignation(
                    &Expr::Member(member_expr),
                    &mut assignations,
                    &mut arguments,
                    &span,
                    IdentKind::Expr,
                )
            }
        };

        arguments.push(ExprOrSpread::from(ident_replacement.clone()));

        let mut call_replacement = replace_call_callee_and_args(
            call,
            Some(ident_callee.map_or_else(|| expr.clone(), Expr::Ident)),
            &mut assignations,
            &mut arguments,
            call_or_apply,
            ident_provider,
        );

        // insert .call(this) argument
        call_replacement.args.insert(
            0,
            ExprOrSpread {
                spread: None,
                expr: Box::new(ident_replacement),
            },
        );

        return Some(ResultExpr {
            tag: method_name.clone(),
            expr: get_dd_paren_expr(
                &Expr::Call(call_replacement),
                &arguments,
                &mut assignations,
                csi_method.dst.as_str(),
                &span,
            ),
        });
    }
    None
}

fn replace_call_expr_or_spread_if_csi_method_with_member(
    expr_or_spread: &ExprOrSpread,
    ident_name: &IdentName,
    call: &mut CallExpr,
    csi_methods: &CsiMethods,
    member_expr: &MemberExpr,
    call_or_apply: &str,
    ident_provider: &mut dyn IdentProvider,
) -> Option<ResultExpr> {
    if expr_or_spread.spread.is_none() {
        //  String.prototype.concat.call(a, b) or a.concat(b)
        replace_call_expr_if_csi_method_with_member(
            &expr_or_spread.expr,
            ident_name,
            call,
            csi_methods,
            Some(member_expr),
            Some(call_or_apply),
            ident_provider,
        )
    } else {
        //  String.prototype.concat.call(...a, b)
        replace_call_spread_if_csi_method_with_member(
            ident_name,
            call,
            csi_methods,
            member_expr,
            call_or_apply,
            ident_provider,
        )
    }
}

fn replace_call_spread_if_csi_method_with_member(
    ident_name: &IdentName,
    call: &mut CallExpr,
    csi_methods: &CsiMethods,
    member_expr: &MemberExpr,
    call_or_apply: &str,
    ident_provider: &mut dyn IdentProvider,
) -> Option<ResultExpr> {
    let method_name = &ident_name.sym.to_string();

    if let Some(csi_method) = csi_methods.get(method_name) {
        let mut assignations = Vec::new();
        let mut arguments = Vec::new();
        let span = call.span;

        // __datadog_token_$i2 = member
        let ident_callee = ident_provider.get_ident_used_in_assignation(
            &Expr::Member(member_expr.clone()),
            &mut assignations,
            &mut arguments,
            &span,
            IdentKind::Expr,
        );

        // return if there is no ident_callee
        ident_callee.as_ref()?;

        let call_replacement = replace_call_callee_and_args(
            call,
            Some(Expr::Ident(ident_callee.unwrap())),
            &mut assignations,
            &mut arguments,
            Some(call_or_apply),
            ident_provider,
        );

        return Some(ResultExpr {
            tag: method_name.clone(),
            expr: get_dd_paren_expr(
                &Expr::Call(call_replacement),
                &arguments,
                &mut assignations,
                csi_method.dst.as_str(),
                &span,
            ),
        });
    }
    None
}

fn replace_call_callee_and_args(
    call: &mut CallExpr,
    ident_callee_expr: Option<Expr>,
    assignations: &mut Vec<Expr>,
    arguments: &mut Vec<ExprOrSpread>,
    call_or_apply: Option<&str>,
    ident_provider: &mut dyn IdentProvider,
) -> CallExpr {
    let mut call_replacement = call.clone();

    let span = call.span;

    let prop_name = call_or_apply.unwrap_or("call");

    // change callee to __datadog_token_$i2.[call|apply]
    if let Some(ident) = ident_callee_expr {
        call_replacement.callee = Callee::Expr(Box::new(Expr::Member(MemberExpr {
            span,
            obj: Box::new(ident),
            prop: MemberProp::Ident(IdentName::new(prop_name.into(), span)),
        })));
    }

    let expand_arrays = if prop_name == "apply" {
        ExpandArrays::Yes
    } else {
        ExpandArrays::No
    };
    call_replacement.args.iter_mut().for_each(|expr_or_spread| {
        DefaultOperandHandler::replace_expressions_in_expr_or_spread(
            expr_or_spread,
            IdentMode::Replace,
            assignations,
            arguments,
            &span,
            ident_provider,
            expand_arrays,
        )
    });

    call_replacement
}
