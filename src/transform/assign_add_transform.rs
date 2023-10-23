/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use swc::ecmascript::ast::*;
use swc_ecma_visit::VisitMutWith;

use crate::{
    transform::assign_add_transform::AssignOp::Assign,
    visitor::operation_transform_visitor::OperationTransformVisitor,
};

use super::{binary_add_transform::BinaryAddTransform, transform_status::TransformResult};

pub struct AssignAddTransform {}

impl AssignAddTransform {
    pub fn to_dd_assign_expr(
        assign: &mut AssignExpr,
        opv: &mut OperationTransformVisitor,
    ) -> TransformResult<AssignExpr> {
        let span = assign.span;

        match &assign.left {
            PatOrExpr::Pat(_) => {
                assign.visit_mut_children_with(opv);
                TransformResult::not_modified()
            }

            PatOrExpr::Expr(left_expr) => {
                let binary = Expr::Bin(BinExpr {
                    span,
                    op: BinaryOp::Add,
                    left: left_expr.clone(),
                    right: assign.right.clone(),
                });

                let result = BinaryAddTransform::to_dd_binary_expr(
                    &binary,
                    opv.csi_methods,
                    opv.ident_provider,
                );
                if result.is_modified() {
                    let new_assign = AssignExpr {
                        span,
                        op: Assign,
                        left: assign.left.clone(),
                        right: Box::new(result.expr.unwrap()),
                    };
                    TransformResult::modified(new_assign)
                } else {
                    TransformResult::not_modified()
                }
            }
        }
    }
}
