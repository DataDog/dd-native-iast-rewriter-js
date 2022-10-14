/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use std::collections::HashSet;
use swc::{common::util::take::Take, ecmascript::ast::*};
use swc_ecma_visit::{Visit, VisitMut, VisitMutWith};

use super::{
    assign_add_transform::AssignAddTransform,
    binary_add_transform::BinaryAddTransform,
    ident_provider::IdentProvider,
    template_transform::TemplateTransform,
    transform_status::TransformStatus,
    visitor_with_context::{Ctx, VisitorWithContext, WithCtx},
};

pub struct OperationTransformVisitor {
    pub ident_counter: usize,
    pub idents: Vec<Ident>,
    pub variable_decl: HashSet<Ident>,
    pub transform_status: TransformStatus,
    ctx: Ctx,
}

impl OperationTransformVisitor {
    pub fn new() -> Self {
        OperationTransformVisitor {
            ident_counter: 0,
            idents: Vec::new(),
            variable_decl: HashSet::new(),
            transform_status: TransformStatus::not_modified(),
            ctx: Ctx::root(),
        }
    }

    fn with_ctx(&mut self, ctx: Ctx) -> WithCtx<'_, OperationTransformVisitor> {
        let orig_ctx = self.ctx;
        self.ctx = ctx;
        WithCtx {
            reducer: self,
            orig_ctx,
        }
    }

    fn with_child_ctx(&mut self) -> WithCtx<'_, OperationTransformVisitor> {
        self.with_ctx(self.ctx.child(false))
    }
}

impl VisitorWithContext for OperationTransformVisitor {
    fn get_ctx(&self) -> Ctx {
        self.ctx
    }

    fn set_ctx(&mut self, ctx: Ctx) {
        self.ctx = ctx;
    }

    fn reset_ctx(&mut self) {
        if self.ctx.root {
            self.ident_counter = 0;
        }
    }
}

impl IdentProvider for OperationTransformVisitor {
    fn register_ident(&mut self, ident: Ident) {
        if !self.idents.contains(&ident) {
            self.idents.push(ident);
        }
    }

    fn next_ident(&mut self) -> usize {
        let counter = self.ident_counter;
        self.ident_counter += 1;
        counter
    }

    fn set_status(&mut self, status: TransformStatus) {
        self.transform_status = status;
    }
}

impl Visit for OperationTransformVisitor {}

impl VisitMut for OperationTransformVisitor {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Bin(binary) => {
                binary.visit_mut_children_with(&mut *self.with_child_ctx());
                if binary.op == BinaryOp::Add {
                    expr.map_with_mut(|bin| BinaryAddTransform::to_dd_binary_expr(&bin, self));
                }
                self.reset_ctx();
            }
            Expr::Assign(assign) => {
                assign.visit_mut_children_with(&mut *self.with_child_ctx());
                if assign.op == AssignOp::AddAssign {
                    assign.map_with_mut(|mut assign| {
                        AssignAddTransform::to_dd_assign_expr(&mut assign, self)
                    });
                }
                self.reset_ctx();
            }
            Expr::Tpl(tpl) => {
                if !tpl.exprs.is_empty() {
                    // transform tpl into binary and act like it was a binary expr
                    let mut binary = TemplateTransform::get_binary_from_tpl(tpl);
                    binary.visit_mut_children_with(&mut *self.with_child_ctx());
                    expr.map_with_mut(|_| BinaryAddTransform::to_dd_binary_expr(&binary, self));
                    self.reset_ctx();
                }
            }
            _ => {
                expr.visit_mut_children_with(self);
            }
        }
    }

    fn visit_mut_ident(&mut self, ident: &mut Ident) {
        self.variable_decl.insert(ident.clone());
    }

    fn visit_mut_if_stmt(&mut self, if_stmt: &mut IfStmt) {
        if_stmt.test.visit_mut_children_with(self);
        if_stmt.cons.visit_mut_children_with(self);
    }

    // cancel visit child blocks
    fn visit_mut_block_stmt(&mut self, _n: &mut BlockStmt) {}
}
