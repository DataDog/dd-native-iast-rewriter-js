use crate::visitor::{
    assign_transform_visitor::AssignTransformVisitor,
    operation_transform_visitor::OperationTransformVisitor,
};
use swc::ecmascript::ast::{Stmt::Decl as DeclEnumOption, *};
use swc_ecma_visit::{Visit, VisitMut, VisitMutWith};

pub struct BlockTransformVisitor {}

//  new algorithm
//  Block:
//  - Find items to instrument (+ or template literals in statements or in while, if... test part)
//  - Replace found items by (__dd_XXX_1=....)
//  - Create necessary temporal vars in top of block (improve it in the future forcing deletion)

impl Visit for BlockTransformVisitor {}

impl VisitMut for BlockTransformVisitor {
    fn visit_mut_block_stmt(&mut self, expr: &mut BlockStmt) {
        let operation_visitor = &mut OperationTransformVisitor {
            assign_visitor: AssignTransformVisitor {},
            idents: Vec::new(),
        };
        expr.visit_mut_children_with(operation_visitor);
        insert_var_declaration(&operation_visitor.idents, expr);
        expr.visit_mut_children_with(self);
    }
}

fn insert_var_declaration(ident_expressions: &Vec<Ident>, expr: &mut BlockStmt) {
    if ident_expressions.len() > 0 {
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
        expr.stmts.insert(0, declaration);
    }
}
