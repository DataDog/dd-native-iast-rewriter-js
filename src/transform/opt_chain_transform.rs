/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use crate::visitor::{
    csi_methods::CsiMethods,
    ident_provider::{IdentKind, IdentProvider},
};
use swc::atoms::JsWord;
use swc_common::{util::take::Take, SyntaxContext, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_visit::{Visit, VisitMut, VisitMutWith};

use super::transform_status::TransformResult;

struct OptChainVisitor<'a> {
    pub assignment: Option<Expr>,
    pub assignments: Vec<Expr>,
    pub new_ident: Option<Ident>,
    pub ident_provider: &'a mut dyn IdentProvider,
    pub csi_methods: &'a CsiMethods,
    pub found: bool,
}

impl OptChainVisitor<'_> {
    pub fn default<'a>(
        ident_provider: &'a mut dyn IdentProvider,
        csi_methods: &'a CsiMethods,
    ) -> OptChainVisitor<'a> {
        OptChainVisitor {
            assignment: None,
            assignments: Vec::new(),
            new_ident: None,
            ident_provider,
            csi_methods,
            found: false,
        }
    }

    fn get_call_from_base_call(&mut self, call_expr: &OptCall, optional: bool) -> Option<CallExpr> {
        if optional {
            /*
             * if expression is like obj.b?(arg1, arg2), to prevent double calling to the b getter
             *  we should extract obj.b to a variable, and call to the new variable maintaining
             *  the expected this property:
             *
             *  (_1 = obj, _2 = _1.b, _2 == null ? undefined : _2.call(_1, arg1, arg2))
             */
            if let Expr::Member(mut member_expr) = *call_expr.callee.clone() {
                let mut member_obj_arguments = Vec::new();
                let span = DUMMY_SP;
                let member_obj_ident_opt = self.ident_provider.get_ident_used_in_assignation(
                    &member_expr.obj.clone(),
                    &mut self.assignments,
                    &mut member_obj_arguments,
                    &span,
                    IdentKind::Expr,
                );

                if let Some(member_obj_ident) = member_obj_ident_opt {
                    let new_member_expr = MemberExpr {
                        span: DUMMY_SP,
                        obj: Box::new(Expr::Ident(member_obj_ident.clone())),
                        prop: member_expr.prop.clone(),
                    };

                    member_expr.map_with_mut(|_| new_member_expr);

                    let mut member_expr_args = Vec::new();
                    let member_expr_ident_opt = self.ident_provider.get_ident_used_in_assignation(
                        &Expr::Member(member_expr.clone()),
                        &mut self.assignments,
                        &mut member_expr_args,
                        &span,
                        IdentKind::Expr,
                    );

                    if let Some(member_expr_ident) = member_expr_ident_opt {
                        self.new_ident = Some(member_expr_ident.clone());
                        let call_ident = Ident {
                            span: DUMMY_SP,
                            sym: "call".into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        };
                        let callee = MemberExpr {
                            span: DUMMY_SP,
                            obj: Box::new(Expr::Ident(member_expr_ident)),
                            prop: MemberProp::Ident(IdentName::from(call_ident)),
                        };

                        let mut args = Vec::new();
                        args.push(ExprOrSpread {
                            expr: Box::new(Expr::Ident(member_obj_ident)),
                            spread: None,
                        });
                        for arg in call_expr.clone().args {
                            args.push(arg)
                        }
                        let call_expr_new = CallExpr {
                            span: DUMMY_SP,
                            callee: Callee::Expr(Box::new(Expr::Member(callee))),
                            args,
                            ctxt: call_expr.ctxt,
                            type_args: call_expr.type_args.clone(),
                        };

                        return Some(call_expr_new);
                    }
                }
            } else {
                let mut arguments = Vec::new();
                let span = DUMMY_SP;
                let new_ident_opt = self.ident_provider.get_ident_used_in_assignation(
                    &call_expr.callee.clone(),
                    &mut self.assignments,
                    &mut arguments,
                    &span,
                    IdentKind::Expr,
                );

                if let Some(new_ident) = new_ident_opt {
                    if let Some(fist_arg) = self.assignments.first() {
                        self.assignment = Some(fist_arg.clone());
                        self.new_ident = Some(new_ident.clone());
                        let call_expr_new = CallExpr {
                            span: DUMMY_SP,
                            callee: Callee::Expr(Box::new(Expr::Ident(new_ident))),
                            args: call_expr.args.clone(),
                            ctxt: call_expr.ctxt,
                            type_args: call_expr.type_args.clone(),
                        };

                        return Some(call_expr_new);
                    }
                }
            }
        } else {
            return Some(CallExpr {
                span: DUMMY_SP,
                callee: call_expr.callee.clone().into(),
                args: call_expr.args.clone(),
                ctxt: call_expr.ctxt,
                type_args: call_expr.type_args.clone(),
            });
        }
        None
    }

    fn get_member_from_base_member(
        &mut self,
        member_expr: &MemberExpr,
        optional: bool,
    ) -> Option<MemberExpr> {
        if optional {
            let mut arguments = Vec::new();
            let span = DUMMY_SP;
            let new_ident_opt = self.ident_provider.get_ident_used_in_assignation(
                &member_expr.obj.clone(),
                &mut self.assignments,
                &mut arguments,
                &span,
                IdentKind::Expr,
            );

            if let Some(new_ident) = new_ident_opt {
                self.new_ident = Some(new_ident.clone());

                let member_expr_new = MemberExpr {
                    span: DUMMY_SP,
                    obj: Box::new(Expr::Ident(new_ident)),
                    prop: member_expr.prop.clone(),
                };

                return Some(member_expr_new);
            }
        } else {
            return Some(member_expr.clone());
        }

        None
    }
}

impl Visit for OptChainVisitor<'_> {}

impl VisitMut for OptChainVisitor<'_> {
    /*
     * Iterates the OptChain finding a method to replace.
     *  If the expression contains method to rewrite, all the OptCall or OptMembers are converted
     *  normal Member or Call expressions, and the optional part of the expression is extracted
     *  to a new variable.
     */
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        match expr {
            Expr::OptChain(opt_chain_expr) => {
                if self.found {
                    let base = &*opt_chain_expr.base;
                    let optional = opt_chain_expr.optional;
                    match base {
                        OptChainBase::Call(call_expr) => {
                            let call_opt = self.get_call_from_base_call(call_expr, optional);
                            if let Some(call) = call_opt {
                                expr.map_with_mut(|_| Expr::Call(call))
                            }
                        }

                        OptChainBase::Member(member_expr) => {
                            let call_opt = self.get_member_from_base_member(member_expr, optional);
                            if let Some(call) = call_opt {
                                expr.map_with_mut(|_| Expr::Member(call))
                            }
                        }
                    };

                    if optional {
                        // Do not call to visit_mut_children_with
                        return;
                    }
                } else if !opt_chain_expr.optional {
                    if let OptChainBase::Call(opt_call) = &*opt_chain_expr.base {
                        if let Expr::OptChain(opt_chain_expr) = *opt_call.clone().callee {
                            if let OptChainBase::Member(member_expr) = &*opt_chain_expr.base {
                                if let MemberProp::Ident(method_ident) = &member_expr.prop {
                                    let prop_name = &method_ident.sym;

                                    if self.csi_methods.get(prop_name).is_some() {
                                        self.found = true;

                                        expr.visit_mut_with(self);
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }

                expr.visit_mut_children_with(self);
            }

            _ => {
                expr.visit_mut_children_with(self);
            }
        };
    }
}

pub struct OptChainTransform {}

impl OptChainTransform {
    pub fn to_dd_cond_expr(
        opt_chain_expr: &mut Expr,
        csi_methods: &CsiMethods,
        ident_provider: &mut dyn IdentProvider,
    ) -> TransformResult<Expr> {
        let visitor = &mut OptChainVisitor::default(ident_provider, csi_methods);
        opt_chain_expr.visit_mut_with(visitor);

        // If the optional chaining contains a method to rewrite, we should modify the expression
        //  by a paren expressions assigning the extracted part to a variable and checking if it
        //  is null to return undefined
        //  (extracted_var = optional_part, extracted_var == null ? undefined : extracted_var.the_rest_of_the_expression)
        if visitor.assignments.is_empty() || visitor.new_ident.is_none() {
            return TransformResult::not_modified();
        }

        let new_ident = visitor.new_ident.as_mut().unwrap();

        let test = Expr::Bin(BinExpr {
            span: DUMMY_SP,
            op: BinaryOp::EqEq,
            left: Box::new(Expr::Ident(new_ident.clone())),
            right: Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))),
        });

        let cons = Ident {
            span: DUMMY_SP,
            sym: JsWord::from("undefined"),
            optional: false,
            ctxt: SyntaxContext::empty(),
        };

        let cond = CondExpr {
            span: DUMMY_SP,
            test: Box::new(test),
            cons: Box::new(Expr::Ident(cons)),
            alt: Box::new(opt_chain_expr.clone()),
        };

        visitor.assignments.push(Expr::Cond(cond));

        let expr = Expr::Paren(ParenExpr {
            span: DUMMY_SP,
            expr: Box::new(Expr::Seq(SeqExpr {
                span: DUMMY_SP,
                exprs: visitor
                    .assignments
                    .iter_mut()
                    .map(|assignation| Box::new(std::mem::take(assignation)))
                    .collect::<Vec<Box<Expr>>>(),
            })),
        });

        opt_chain_expr.map_with_mut(|_| expr);

        TransformResult::modified(opt_chain_expr.clone())
    }
}
