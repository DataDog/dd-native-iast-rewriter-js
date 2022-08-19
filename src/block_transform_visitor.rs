use swc_ecma_visit::{Visit, VisitMut, VisitMutWith};
// use swc::ecmascript::ast::*;
use swc::atoms::JsWord;
use swc::ecmascript::ast::*;//{BindingIdent, BlockStmt, Pat, VarDeclarator, Ident, VarDecl, VarDeclKind};
use swc::ecmascript::ast::Stmt::Decl as DeclEnumOption;
use crate::operation_transform_visitor::OperationTransformVisitor;

pub struct BlockTransformVisitor {}

// TODO: new algorithm
//  Block:
//  - Find items to instrument (+ or template literals in statements or in while, if... test part)
//  - Replace found items by (__dd_XXX_1=....)
//  - Create necessary temporal vars in top of block (improve it in the future forcing deletion)

impl Visit for BlockTransformVisitor {}

impl VisitMut for BlockTransformVisitor {
    fn visit_mut_block_stmt(&mut self, expr: &mut BlockStmt) {
        println!("block {:#?}", expr);
        let operation_visitor = &mut OperationTransformVisitor { counter: 0 };
        expr.visit_mut_children_with(operation_visitor);
        insert_var_declaration(operation_visitor.counter, expr);
        expr.visit_mut_children_with(self);
        // expr.stmts.clear();
    }
}

fn insert_var_declaration(counter: i32, expr: &mut BlockStmt) {
    if counter > 0 {
        let span = expr.span;
        let mut vec = Vec::new();
        for n in 0 .. counter {
            let var_declarator = VarDeclarator {
                span,
                definite: false,
                name: Pat::Ident(BindingIdent {
                    id: Ident {
                        span,
                        sym: JsWord::from(format!("__dd_{}", n)),
                        optional: false
                    },
                    type_ann: None
                }),
                init: None
            };
            vec.push(var_declarator);
        }
        let declaration = DeclEnumOption(Decl::Var(VarDecl {
            span,
            decls: vec,
            declare: false,
            kind: VarDeclKind::Let
        }));
        expr.stmts.insert(0, declaration);
    }
    println!("Inserted vars: {}", counter);
}