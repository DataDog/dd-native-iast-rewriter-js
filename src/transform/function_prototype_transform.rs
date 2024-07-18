use swc_common::SyntaxContext;
/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use swc_ecma_ast::IdentName;
use swc_ecma_visit::swc_ecma_ast::{
    CallExpr, Callee, Expr, ExprOrSpread, Ident, MemberExpr, MemberProp,
};

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
    ///     Expr -> $this_expr,
    ///     Ident -> $method_name,
    ///     CallExpr -> a expression equivalent to $this_expr.$method_name($arguments)
    /// )
    ///
    pub fn get_expression_parts_from_call_or_apply(
        call: &CallExpr,
        member: &MemberExpr,
        ident_name: &IdentName,
    ) -> Option<(Expr, Ident, CallExpr)> {
        if !Self::is_call_or_apply(ident_name) {
            return None;
        }

        let method_name = ident_name.sym.to_string();
        let mut path_parts = vec![];
        if get_prototype_member_path(member, &mut path_parts) {
            if call.args.is_empty() {
                return None;
            }

            let this_expr = &call.args[0].expr;
            if this_expr.is_lit() {
                return None;
            }

            let mut filtered_args = vec![];
            if !filter_call_args(
                &call.args,
                method_name == APPLY_METHOD_NAME,
                &mut filtered_args,
            ) {
                return None;
            }

            let method_ident = path_parts[0].clone();

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

            return Some((*this_expr.clone(), method_ident, new_call));
        }

        None
    }
}

fn filter_call_args(
    args: &[ExprOrSpread],
    is_apply: bool,
    filtered_args: &mut Vec<ExprOrSpread>,
) -> bool {
    // when using apply, arguments are provided as an array
    let mut success_filtering = true;
    if is_apply {
        if args.len() >= 2 {
            if args[1].expr.is_array() {
                let array = args[1].expr.as_array().unwrap();
                filtered_args.append(
                    &mut array
                        .elems
                        .iter()
                        .filter(|elem| elem.is_some())
                        .map(|elem| elem.as_ref().unwrap().clone())
                        .collect(),
                );
            } else {
                success_filtering = false;
            }
        }
    } else {
        filtered_args.append(&mut args.iter().skip(1).cloned().collect::<Vec<ExprOrSpread>>());
    }
    success_filtering
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
