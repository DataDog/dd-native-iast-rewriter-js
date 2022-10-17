/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use crate::visitor::{
    operation_transform_visitor::OperationTransformVisitor,
    visitor_util::get_dd_local_variable_prefix,
};
use std::collections::HashSet;
use swc::ecmascript::ast::{Stmt::Decl as DeclEnumOption, *};
use swc_ecma_visit::{Visit, VisitMut, VisitMutWith};

use super::transform_status::{Status, TransformStatus};

pub struct BlockTransformVisitor<'a> {
    pub transform_status: &'a mut TransformStatus,
}

impl BlockTransformVisitor<'_> {
    pub fn default(transform_status: &mut TransformStatus) -> BlockTransformVisitor<'_> {
        BlockTransformVisitor { transform_status }
    }

    fn visit_is_cancelled(&mut self) -> bool {
        self.transform_status.status == Status::Cancelled
    }

    fn cancel_visit(&mut self, reason: &str) {
        self.transform_status.status = Status::Cancelled;
        self.transform_status.msg = reason.to_string();
    }

    fn mark_modified(&mut self) {
        if !self.visit_is_cancelled() {
            self.transform_status.status = Status::Modified;
        }
    }
}

//  new algorithm
//  Block:
//  - Find items to instrument (+ or template literals in statements or in while, if... test part)
//  - Replace found items by (__dd_XXX_1=....)
//  - Create necessary temporal vars in top of block (improve it in the future forcing deletion)

impl Visit for BlockTransformVisitor<'_> {}

impl VisitMut for BlockTransformVisitor<'_> {
    fn visit_mut_block_stmt(&mut self, expr: &mut BlockStmt) {
        if self.visit_is_cancelled() {
            return;
        }

        let operation_visitor = &mut OperationTransformVisitor::new();
        expr.visit_mut_children_with(operation_visitor);

        if operation_visitor.transform_status.status == Status::Modified {
            self.mark_modified();
        }

        if variables_contains_possible_duplicate(&operation_visitor.variable_decl) {
            return self.cancel_visit("Variable name duplicated");
        }

        insert_var_declaration(&operation_visitor.idents, expr);

        expr.visit_mut_children_with(self);
    }
}

fn variables_contains_possible_duplicate(variable_decl: &HashSet<Ident>) -> bool {
    let prefix = get_dd_local_variable_prefix();
    variable_decl.iter().any(|var| var.sym.starts_with(&prefix))
}

fn insert_var_declaration(ident_expressions: &Vec<Ident>, expr: &mut BlockStmt) {
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
        let declaration = DeclEnumOption(Decl::Var(VarDecl {
            span,
            decls: vec,
            declare: false,
            kind: VarDeclKind::Let,
        }));

        let index = get_variable_insertion_index(&expr.stmts);
        expr.stmts.insert(index, declaration);
    }
}

fn get_variable_insertion_index(stmts: &Vec<Stmt>) -> usize {
    if !stmts.is_empty() {
        match &stmts[0] {
            Stmt::Expr(expr) => match &*expr.expr {
                Expr::Lit(Lit::Str(lit)) => {
                    if lit.value.eq("use strict") {
                        return 1;
                    } else {
                        return 0;
                    }
                }
                _ => return 0,
            },
            _ => return 0,
        }
    }

    0
}
