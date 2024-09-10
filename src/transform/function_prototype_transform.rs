/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use swc_common::{util::take::Take, SyntaxContext, DUMMY_SP};
use swc_ecma_ast::IdentName;
use swc_ecma_visit::swc_ecma_ast::{
    CallExpr, Callee, Expr, ExprOrSpread, Ident, MemberExpr, MemberProp,
};

use crate::visitor::csi_methods::CsiMethods;

pub const PROTOTYPE: &str = "prototype";
pub const CALL_METHOD_NAME: &str = "call";
pub const APPLY_METHOD_NAME: &str = "apply";

pub struct FunctionPrototypeTransform {}

impl FunctionPrototypeTransform {
    pub fn is_call_or_apply(ident_name: &IdentName) -> bool {
        let method_name = ident_name.sym.to_string();
        method_name == CALL_METHOD_NAME || method_name == APPLY_METHOD_NAME
    }

    pub fn member_prop_is_prototype(member: &MemberExpr) -> bool {
        member.prop.is_ident() && member.prop.as_ident().unwrap().sym == PROTOTYPE
    }

    /// inspects call expression searching for $class_name.prototype.$method_name.[call|apply]($this_expr, $arguments) and if there is a match
    /// returns a tuple (
    ///     ExprOrSpread -> $this_expr,
    ///     Ident -> $method_name,
    ///     CallExpr -> an expression equivalent to $this_expr.$method_name($arguments)
    /// )
    ///
    pub fn get_expression_parts_from_call_or_apply(
        call: &CallExpr,
        member: &MemberExpr,
        ident_name: &IdentName,
        csi_methods: &CsiMethods,
    ) -> Option<(ExprOrSpread, Ident, CallExpr)> {
        if !Self::is_call_or_apply(ident_name) {
            return None;
        }

        let mut path_parts = vec![];
        if get_prototype_member_path(member, &mut path_parts) {
            if call.args.is_empty() {
                return None;
            }

            let method_ident = path_parts[0].clone();
            let this_expr_or_spread = &call.args[0];

            // ...$this_expr - we can not return an Expr for an spread expression
            if this_expr_or_spread.spread.is_some() {
                return Some((this_expr_or_spread.clone(), method_ident, call.clone()));
            }

            if invalid_args(ident_name, call) {
                return None;
            }

            let filtered_args = call
                .args
                .iter()
                .skip(1)
                .cloned()
                .collect::<Vec<ExprOrSpread>>();

            let this_expr = &this_expr_or_spread.expr;

            if this_expr.is_lit()
                && (!csi_methods.method_allows_literal_callers(&method_ident.sym)
                    || all_args_are_literal(&filtered_args))
            {
                return None;
            }

            let new_callee = MemberExpr {
                obj: this_expr.clone(),
                prop: MemberProp::Ident(IdentName::new(
                    method_ident.sym.clone(),
                    method_ident.span,
                )),
                span: call.span,
            };

            let new_call = CallExpr {
                args: filtered_args,
                callee: Callee::Expr(Box::new(Expr::Member(new_callee))),
                span: call.span,
                type_args: None,
                ctxt: SyntaxContext::empty(),
            };

            return Some((
                ExprOrSpread::from(*this_expr.clone()),
                method_ident,
                new_call,
            ));
        }

        None
    }

    pub fn get_member_expr_from_path(path: &str) -> MemberExpr {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() < 2 {
            return MemberExpr::dummy();
        }

        let mut member_expr = MemberExpr {
            obj: Box::new(Expr::Ident(Ident::from(parts[0]))),
            prop: MemberProp::Ident(IdentName::new(parts[1].into(), DUMMY_SP)),
            span: DUMMY_SP,
        };

        parts.iter().skip(2).for_each(|part| {
            if part.is_empty() {
                return;
            }

            member_expr = MemberExpr {
                obj: Box::new(Expr::Member(member_expr.clone())),
                prop: MemberProp::Ident(IdentName::new(String::from(*part).into(), DUMMY_SP)),
                span: DUMMY_SP,
            };
        });

        member_expr
    }
}

fn get_prototype_member_path(member: &MemberExpr, parts: &mut Vec<Ident>) -> bool {
    if member.prop.is_ident() {
        let member_prop_ident = member.prop.as_ident().unwrap();
        parts.push(Ident::from(member_prop_ident.clone()));
        if member.obj.is_member() {
            let member_obj_member = member.obj.as_member();
            return get_prototype_member_path(member_obj_member.unwrap(), parts);
        } else if member.obj.is_ident() {
            let last_ident = member.obj.as_ident().unwrap();
            parts.push(last_ident.clone());
        }
    }
    !parts.is_empty()
}

fn all_args_are_literal(args: &[ExprOrSpread]) -> bool {
    args.iter()
        .all(|elem| elem.expr.is_lit() || is_undefined_or_null(&elem.expr))
}

fn is_undefined_or_null(expr: &Expr) -> bool {
    expr.is_ident_ref_to("undefined") || expr.is_ident_ref_to("null")
}

fn invalid_args(ident_name: &IdentName, call: &CallExpr) -> bool {
    let name = ident_name.sym.to_string();
    if name != "apply" {
        return false;
    }

    if call.args.len() >= 2 {
        let this = &call.args[0];
        let args_array = &call.args[1];
        if args_array.expr.is_array() {
            let array = args_array.expr.as_array().unwrap();
            return this.expr.is_lit()
                && array.elems.iter().skip(1).all(|elem| {
                    if elem.is_none() {
                        return false;
                    }

                    let expr_or_spread = elem.as_ref().unwrap();
                    expr_or_spread.expr.is_lit() || is_undefined_or_null(&expr_or_spread.expr)
                });
        } else if args_array.spread.is_some() {
            return false;
        }
    }

    true
}
