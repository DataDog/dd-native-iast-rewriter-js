/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use crate::{
    rewriter::Config,
    transform::transform_status::{Status, TransformStatus},
    visitor::{
        operation_transform_visitor::OperationTransformVisitor,
        visitor_util::get_dd_local_variable_prefix,
    },
};
use std::collections::HashSet;
use swc_common::SyntaxContext;
use swc_ecma_ast::{Stmt::Decl as DeclEnumOption, *};
use swc_ecma_visit::{Visit, VisitMut, VisitMutWith};

use super::{ident_provider::DefaultIdentProvider, visitor_with_context::Ctx};

pub struct BlockTransformVisitor<'a> {
    pub transform_status: &'a mut TransformStatus,
    pub config: &'a Config,
}

impl BlockTransformVisitor<'_> {
    pub fn default<'a>(
        transform_status: &'a mut TransformStatus,
        config: &'a Config,
    ) -> BlockTransformVisitor<'a> {
        BlockTransformVisitor {
            transform_status,
            config,
        }
    }

    fn visit_is_cancelled(&mut self) -> bool {
        self.transform_status.status == Status::Cancelled
    }

    fn cancel_visit(&mut self, reason: &str) {
        self.transform_status.status = Status::Cancelled;
        self.transform_status.msg = Some(reason.to_string());
    }
}

//  Block:
//  - Find items to instrument (+ or template literals in statements or in while, if... test part)
//  - Replace found items by (__dd_XXX_1=....)
//  - Create necessary temporal vars in top of block

impl Visit for BlockTransformVisitor<'_> {}

impl VisitMut for BlockTransformVisitor<'_> {
    fn visit_mut_block_stmt(&mut self, expr: &mut BlockStmt) {
        if self.visit_is_cancelled() {
            return;
        }

        let mut ident_provider = DefaultIdentProvider::new(&self.config.local_var_prefix);
        let mut operation_visitor = OperationTransformVisitor {
            ident_provider: &mut ident_provider,
            csi_methods: &self.config.csi_methods,
            transform_status: self.transform_status,
            ctx: Ctx::root(),
        };

        expr.visit_mut_children_with(&mut operation_visitor);

        if variables_contains_possible_duplicate(
            &ident_provider.variable_decl,
            &self.config.local_var_prefix,
        ) {
            return self.cancel_visit("Variable name duplicated");
        } else {
            insert_variable_declaration(&ident_provider.idents, expr);
        }

        expr.visit_mut_children_with(self);
    }
}

fn variables_contains_possible_duplicate(variable_decl: &HashSet<Ident>, prefix: &String) -> bool {
    let prefix = get_dd_local_variable_prefix(prefix);
    variable_decl.iter().any(|var| var.sym.starts_with(&prefix))
}

fn insert_variable_declaration(ident_expressions: &[Ident], expr: &mut BlockStmt) {
    if !ident_expressions.is_empty() {
        let span = expr.span;
        let mut vec = Vec::new();
        ident_expressions.iter().for_each(|ident| {
            vec.push(VarDeclarator {
                span,
                definite: false,
                name: Pat::Ident(BindingIdent {
                    id: ident.clone(),
                    type_ann: None,
                }),
                init: None,
            });
        });
        let declaration = DeclEnumOption(Decl::Var(Box::new(VarDecl {
            span,
            decls: vec,
            declare: false,
            kind: VarDeclKind::Let,
            ctxt: SyntaxContext::empty(),
        })));

        let index = get_variable_insertion_index(&expr.stmts);
        expr.stmts.insert(index, declaration);
    }
}

fn get_variable_insertion_index(stmts: &[Stmt]) -> usize {
    if !stmts.is_empty() {
        match &stmts[0] {
            Stmt::Expr(expr) => match &*expr.expr {
                Expr::Lit(Lit::Str(lit)) => {
                    if lit.value.eq("use strict") {
                        return 1;
                    }
                }
                _ => return 0,
            },
            _ => return 0,
        }
    }

    0
}
