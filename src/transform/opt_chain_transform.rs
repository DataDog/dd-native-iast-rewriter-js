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
    pub csi_methods: CsiMethods,
    pub found: bool,
}

impl OptChainVisitor<'_> {
    pub fn default(
        ident_provider: &mut dyn IdentProvider,
        csi_methods: CsiMethods,
    ) -> OptChainVisitor {
        OptChainVisitor {
            assignment: None,
            assignments: Vec::new(),
            new_ident: None,
            ident_provider,
            csi_methods,
            found: false,
        }
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
                    if opt_chain_expr.optional {
                        match &*opt_chain_expr.base {
                            OptChainBase::Call(call_expr) => {
                                /*
                                 * if expression is like obj.b?(arg1, arg2), to prevent double calling to the b getter
                                 *  we should extract a.b to a variable, and call to the new variable maintaining
                                 *  the expected this property:
                                 *
                                 *  (_1 = obj, _2 = _1.b, _2 == null ? undefined : _2.call(_1, arg1, arg2))
                                 */
                                if let Expr::Member(mut member_expr) = *call_expr.callee.clone() {
                                    let mut member_obj_arguments = Vec::new();
                                    let span = DUMMY_SP;
                                    let member_obj_ident_opt =
                                        self.ident_provider.get_ident_used_in_assignation(
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
                                        let member_expr_ident_opt =
                                            self.ident_provider.get_ident_used_in_assignation(
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
                                                prop: MemberProp::Ident(IdentName::from(
                                                    call_ident,
                                                )),
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
                                                callee: Callee::Expr(Box::new(Expr::Member(
                                                    callee,
                                                ))),
                                                args,
                                                ctxt: call_expr.ctxt,
                                                type_args: call_expr.type_args.clone(),
                                            };

                                            expr.map_with_mut(|_| Expr::Call(call_expr_new));
                                        }
                                    }
                                } else {
                                    let mut arguments = Vec::new();
                                    let span = DUMMY_SP;
                                    let new_ident_opt =
                                        self.ident_provider.get_ident_used_in_assignation(
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
                                                callee: Callee::Expr(Box::new(Expr::Ident(
                                                    new_ident,
                                                ))),
                                                args: call_expr.args.clone(),
                                                ctxt: call_expr.ctxt,
                                                type_args: call_expr.type_args.clone(),
                                            };

                                            expr.map_with_mut(|_| Expr::Call(call_expr_new));
                                        }
                                    }
                                }
                            }

                            OptChainBase::Member(member_expr) => {
                                let mut arguments = Vec::new();
                                let span = DUMMY_SP;
                                let new_ident_opt =
                                    self.ident_provider.get_ident_used_in_assignation(
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

                                    expr.map_with_mut(|_| Expr::Member(member_expr_new));
                                }
                            }
                        };

                        // Do not call to visit_mut_children_with
                        return;
                    } else {
                        match &*opt_chain_expr.base.clone() {
                            OptChainBase::Call(base) => {
                                let call_expr = CallExpr {
                                    span: DUMMY_SP,
                                    callee: base.callee.clone().into(),
                                    args: base.args.clone(),
                                    ctxt: base.ctxt,
                                    type_args: base.type_args.clone(),
                                };
                                expr.map_with_mut(|_| Expr::Call(call_expr));
                            }

                            OptChainBase::Member(base) => {
                                expr.map_with_mut(|_| Expr::Member(base.clone()));
                            }
                        };
                    }
                } else if !opt_chain_expr.optional {
                    if let OptChainBase::Call(opt_call) = &mut *opt_chain_expr.base {
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
        opt_chain_expr: &mut OptChainExpr,
        csi_methods: &CsiMethods,
        ident_provider: &mut dyn IdentProvider,
    ) -> TransformResult<Expr> {
        let visitor = &mut OptChainVisitor::default(ident_provider, csi_methods.clone());
        let mut expr_opt_chain_expr = Expr::OptChain(opt_chain_expr.clone());
        expr_opt_chain_expr.visit_mut_with(visitor);

        // If the optional chaining contains a method to rewrite, we should modify the expression
        //  by a paren expressions assigning the extracted part to a variable and checking if it
        //  is null to return undefined
        //  (extracted_var = optional_part, extracted_var == null ? undefined : extracted_var.the_rest_of_the_expression)
        if !visitor.assignments.is_empty() {
            if let Some(new_ident) = &mut visitor.new_ident {
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

                let alt = expr_opt_chain_expr;

                let cond = CondExpr {
                    span: DUMMY_SP,
                    test: Box::new(test),
                    cons: Box::new(Expr::Ident(cons)),
                    alt: Box::new(alt),
                };

                let mut exprs = Vec::new();
                for assignment in visitor.assignments.clone() {
                    exprs.push(assignment.clone());
                }
                exprs.push(Expr::Cond(cond));

                let expr = Expr::Paren(ParenExpr {
                    span: opt_chain_expr.span,
                    expr: Box::new(Expr::Seq(SeqExpr {
                        span: DUMMY_SP,
                        exprs: exprs
                            .iter()
                            .map(|assignation| Box::new(assignation.clone()))
                            .collect::<Vec<Box<Expr>>>(),
                    })),
                });

                let mut opt_chain_expr_clone = Expr::OptChain(opt_chain_expr.clone());
                opt_chain_expr_clone.map_with_mut(|_| expr);

                return TransformResult::modified(opt_chain_expr_clone);
            }
        }

        TransformResult::not_modified()
    }
}
