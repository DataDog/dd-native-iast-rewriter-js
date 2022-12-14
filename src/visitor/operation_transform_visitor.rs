/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use swc::{common::util::take::Take, ecmascript::ast::*};
use swc_ecma_visit::{Visit, VisitMut, VisitMutWith};

use crate::transform::{
    assign_add_transform::AssignAddTransform, binary_add_transform::BinaryAddTransform,
    template_transform::TemplateTransform,
};

use super::{
    csi_methods::CsiMethods,
    ident_provider::IdentProvider,
    no_plus_operator_visitor::NoPlusOperatorVisitor,
    visitor_with_context::{Ctx, VisitorWithContext},
};

pub struct OperationTransformVisitor<'a> {
    pub ident_provider: &'a mut dyn IdentProvider,
    pub csi_methods: &'a CsiMethods,
    pub ctx: Ctx,
}

impl VisitorWithContext for OperationTransformVisitor<'_> {
    fn get_ctx(&self) -> Ctx {
        self.ctx
    }

    fn set_ctx(&mut self, ctx: Ctx) {
        self.ctx = ctx;
    }

    fn reset_ctx(&mut self) {
        if self.ctx.root {
            self.ident_provider.reset_counter();
        }
    }
}

impl Visit for OperationTransformVisitor<'_> {}

impl VisitMut for OperationTransformVisitor<'_> {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Bin(binary) => {
                // check WithCtx::drop. It calls reset_ctx() method when it is destructed
                let opv_with_child_ctx = &mut *self.with_child_ctx();
                binary.visit_mut_children_with(opv_with_child_ctx);
                if binary.op == BinaryOp::Add {
                    expr.map_with_mut(|bin| {
                        BinaryAddTransform::to_dd_binary_expr(
                            &bin,
                            opv_with_child_ctx.csi_methods,
                            opv_with_child_ctx.ident_provider,
                        )
                    });
                }
            }
            Expr::Assign(assign) => {
                let opv_with_child_ctx = &mut *self.with_child_ctx();
                assign.visit_mut_children_with(opv_with_child_ctx);
                if assign.op == AssignOp::AddAssign {
                    assign.map_with_mut(|mut assign| {
                        AssignAddTransform::to_dd_assign_expr(&mut assign, opv_with_child_ctx)
                    });
                }
            }
            Expr::Tpl(tpl) => {
                if !tpl.exprs.is_empty() {
                    // transform tpl into binary and act as if it were a binary expr
                    let mut binary = TemplateTransform::get_binary_from_tpl(tpl);
                    let opv_with_child_ctx = &mut *self.with_child_ctx();
                    binary.visit_mut_children_with(opv_with_child_ctx);
                    expr.map_with_mut(|_| {
                        BinaryAddTransform::to_dd_binary_expr(
                            &binary,
                            opv_with_child_ctx.csi_methods,
                            opv_with_child_ctx.ident_provider,
                        )
                    });
                }
            }
            Expr::Call(call) => {
                let opv_with_child_ctx = &mut *self.with_child_ctx();
                call.visit_mut_children_with(opv_with_child_ctx);
                if let Some(method) = NoPlusOperatorVisitor::get_dd_call_expr(
                    call,
                    opv_with_child_ctx.csi_methods,
                    opv_with_child_ctx.ident_provider,
                ) {
                    expr.map_with_mut(|_| method);
                }
            }
            _ => {
                expr.visit_mut_children_with(self);
            }
        }
    }

    fn visit_mut_ident(&mut self, ident: &mut Ident) {
        self.ident_provider.register_variable(ident);
    }

    fn visit_mut_if_stmt(&mut self, if_stmt: &mut IfStmt) {
        if_stmt.test.visit_mut_children_with(self);
        if_stmt.cons.visit_mut_children_with(self);
    }

    // cancel visit child blocks
    fn visit_mut_block_stmt(&mut self, _n: &mut BlockStmt) {}
}
