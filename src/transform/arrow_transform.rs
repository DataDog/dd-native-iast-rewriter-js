/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use swc_common::{SyntaxContext, DUMMY_SP};
use swc_ecma_ast::*;

use super::transform_status::TransformResult;

pub struct ArrowTransform {}

impl ArrowTransform {
    pub fn to_dd_arrow_expr(arrow: &ArrowExpr) -> TransformResult<Expr> {
        if arrow.body.is_expr() {
            let mut arrow_block = arrow.clone();

            let return_stmt = ReturnStmt {
                span: DUMMY_SP,
                arg: Some(Box::new(*arrow_block.body.expr().unwrap())),
            };

            arrow_block.body = Box::new(BlockStmtOrExpr::BlockStmt(BlockStmt {
                span: DUMMY_SP,
                ctxt: SyntaxContext::empty(),
                stmts: vec![Stmt::Return(return_stmt)],
            }));

            return TransformResult::modified(Expr::Arrow(arrow_block));
        }

        TransformResult::not_modified()
    }
}
