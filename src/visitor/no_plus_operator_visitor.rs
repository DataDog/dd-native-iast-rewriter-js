/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use swc::{common::util::take::Take, ecmascript::ast::*};
use swc_ecma_visit::{Visit, VisitMut, VisitMutWith};

use crate::transform::call_expr_transform::CallExprTransform;

use super::{
    csi_methods::CsiMethods,
    ident_provider::IdentProvider,
    visitor_with_context::{Ctx, VisitorWithContext},
};

pub struct NoPlusOperatorVisitor<'a> {
    pub ident_provider: &'a mut dyn IdentProvider,
    pub csi_methods: &'a CsiMethods,
    pub ctx: Ctx,
}

impl<'a> NoPlusOperatorVisitor<'a> {
    pub fn get_dd_call_expr(
        call: &mut CallExpr,
        csi_methods: &CsiMethods,
        ident_provider: &mut dyn IdentProvider,
    ) -> Option<Expr> {
        if call.callee.is_expr() {
            return CallExprTransform::to_dd_call_expr(call, csi_methods, ident_provider);
        }
        None
    }
}

impl VisitorWithContext for NoPlusOperatorVisitor<'_> {
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

impl Visit for NoPlusOperatorVisitor<'_> {}

impl VisitMut for NoPlusOperatorVisitor<'_> {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        match expr {
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
            _ => expr.visit_mut_children_with(self),
        }
    }
}
