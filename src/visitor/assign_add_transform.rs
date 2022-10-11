/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use swc::ecmascript::ast::*;
use swc_ecma_visit::VisitMutWith;

use crate::visitor::assign_add_transform::AssignOp::Assign;

use super::{
    binary_add_transform::BinaryAddTransform,
    operation_transform_visitor::OperationTransformVisitor,
};

pub struct AssignAddTransform {}

impl AssignAddTransform {
    pub fn to_dd_assign_expr(
        assign: &mut AssignExpr,
        opv: &mut OperationTransformVisitor,
    ) -> AssignExpr {
        let span = assign.span;
        let op = assign.op;

        match &assign.left {
            PatOrExpr::Pat(_) => {
                assign.visit_mut_children_with(opv);
                AssignExpr {
                    span,
                    op,
                    left: assign.left.clone(),
                    right: assign.right.clone(),
                }
            }
            PatOrExpr::Expr(left_expr) => {
                let binary = Expr::Bin(BinExpr {
                    span,
                    op: BinaryOp::Add,
                    left: left_expr.clone(),
                    right: assign.right.clone(),
                });

                AssignExpr {
                    span,
                    op: Assign,
                    left: assign.left.clone(),
                    right: Box::new(BinaryAddTransform::to_dd_binary_expr(&binary, opv)),
                }
            }
        }
    }
}
