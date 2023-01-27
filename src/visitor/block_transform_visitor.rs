/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use crate::{
    transform::transform_status::{Status, TransformStatus},
    visitor::{
        operation_transform_visitor::OperationTransformVisitor,
        visitor_util::get_dd_local_variable_prefix,
    },
};
use std::collections::HashSet;
use swc::ecmascript::ast::{Stmt::Decl as DeclEnumOption, *};
use swc_ecma_visit::{Visit, VisitMut, VisitMutWith};

use super::{
    csi_methods::CsiMethods,
    ident_provider::{DefaultIdentProvider, IdentProvider},
    no_plus_operator_visitor::NoPlusOperatorVisitor,
    visitor_with_context::Ctx,
};

pub struct BlockTransformVisitor<'a> {
    pub transform_status: &'a mut TransformStatus,
    pub local_var_prefix: String,
    csi_methods: &'a CsiMethods,
}

impl BlockTransformVisitor<'_> {
    pub fn default<'a>(
        transform_status: &'a mut TransformStatus,
        local_var_prefix: String,
        csi_methods: &'a CsiMethods,
    ) -> BlockTransformVisitor<'a> {
        BlockTransformVisitor {
            transform_status,
            local_var_prefix,
            csi_methods,
        }
    }

    fn visit_is_cancelled(&mut self) -> bool {
        self.transform_status.status == Status::Cancelled
    }

    fn cancel_visit(&mut self, reason: &str) {
        self.transform_status.status = Status::Cancelled;
        self.transform_status.msg = reason.to_string();
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

        let mut ident_provider =
            DefaultIdentProvider::new(&self.local_var_prefix, self.transform_status);
        expr.visit_mut_children_with(&mut get_visitor(&mut ident_provider, self.csi_methods));

        if variables_contains_possible_duplicate(
            &ident_provider.variable_decl,
            &self.local_var_prefix,
        ) {
            return self.cancel_visit("Variable name duplicated");
        } else {
            insert_variable_declaration(&ident_provider.idents, expr);
        }

        expr.visit_mut_children_with(self);
    }
}

fn get_visitor<'a>(
    ident_provider: &'a mut dyn IdentProvider,
    csi_methods: &'a CsiMethods,
) -> Box<dyn VisitMut + 'a> {
    if csi_methods.plus_operator_is_enabled() {
        Box::new(OperationTransformVisitor {
            ident_provider,
            csi_methods,
            ctx: Ctx::root(),
        })
    } else {
        Box::new(NoPlusOperatorVisitor {
            ident_provider,
            csi_methods,
            ctx: Ctx::root(),
        })
    }
}

fn variables_contains_possible_duplicate(variable_decl: &HashSet<Ident>, prefix: &String) -> bool {
    let prefix = get_dd_local_variable_prefix(prefix);
    variable_decl.iter().any(|var| var.sym.starts_with(&prefix))
}

fn insert_variable_declaration(ident_expressions: &Vec<Ident>, expr: &mut BlockStmt) {
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
                    }
                }
                _ => return 0,
            },
            _ => return 0,
        }
    }

    0
}
