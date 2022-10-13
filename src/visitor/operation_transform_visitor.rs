/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use std::collections::HashSet;
use swc::{
    common::{util::take::Take, Span},
    ecmascript::ast::*,
};
use swc_ecma_visit::{Visit, VisitMut, VisitMutWith};

use super::{
    assign_add_transform::AssignAddTransform,
    binary_add_transform::BinaryAddTransform,
    template_transform::TemplateTransform,
    visitor_util::create_assign_expression,
    visitor_with_context::{Ctx, VisitorWithContext, WithCtx},
};

pub struct OperationTransformVisitor {
    pub ident_counter: usize,
    pub idents: Vec<Ident>,
    pub variable_decl: HashSet<Ident>,
    ctx: Ctx,
}

impl OperationTransformVisitor {
    pub fn new() -> Self {
        OperationTransformVisitor {
            ident_counter: 0,
            idents: Vec::new(),
            variable_decl: HashSet::new(),
            ctx: Ctx::root(),
        }
    }

    pub fn next_ident(&mut self) -> usize {
        let counter = self.ident_counter;
        self.ident_counter += 1;
        counter
    }

    pub fn get_ident_used_in_assignation(
        &mut self,
        operand: Expr,
        assignations: &mut Vec<Expr>,
        arguments: &mut Vec<Expr>,
        span: Span,
    ) -> Ident {
        self.get_ident_used_in_assignation_with_definitive(
            operand,
            assignations,
            arguments,
            span,
            true,
        )
    }

    pub fn get_ident_used_in_assignation_with_definitive(
        &mut self,
        operand: Expr,
        assignations: &mut Vec<Expr>,
        arguments: &mut Vec<Expr>,
        span: Span,
        definitive: bool,
    ) -> Ident {
        let (assign, id) = create_assign_expression(self.next_ident(), operand, span);

        // store ident and assignation expression
        let id_clone = id.to_owned();
        if definitive && !self.idents.contains(&id_clone) {
            self.idents.push(id_clone);
        }

        assignations.push(Expr::Assign(assign));

        // store ident as argument
        arguments.push(Expr::Ident(id.clone()));

        id
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

    // TODO: make it work
    // this function is in charge of calling visit_mut_children_with creating a new child Ctx, calling delegate function and reseting ctx at the end.
    // It should be used in the different match cases but it changes the behavior of the block codes. I guess the delegate expr object is cloned
    // and expr.map_with_mut() has no effect in the real AST.
    #[allow(dead_code)]
    fn visit_mut_children_with_child_ctx<E, F>(&mut self, expr: &mut E, op: F)
    where
        E: VisitMutWith<OperationTransformVisitor>,
        F: FnOnce(&mut E, &mut OperationTransformVisitor),
    {
        expr.visit_mut_children_with(&mut *self.with_child_ctx());
        op(expr, self);
        self.reset_ctx();
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
        self.variable_decl.insert(ident.to_owned());
    }

    fn visit_mut_if_stmt(&mut self, if_stmt: &mut IfStmt) {
        if_stmt.test.visit_mut_children_with(self);
        if_stmt.cons.visit_mut_children_with(self);
    }

    // cancel visit child blocks
    fn visit_mut_block_stmt(&mut self, _n: &mut BlockStmt) {}
}
