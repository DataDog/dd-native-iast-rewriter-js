/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
use swc_common::Spanned;
use swc_ecma_ast::*;

use crate::visitor::{
    csi_methods::CsiMethods,
    ident_provider::{IdentKind, IdentProvider},
    visitor_util::get_dd_paren_expr,
};

use super::{
    operand_handler::{DefaultOperandHandler, ExpandArrays, IdentMode, OperandHandler},
    transform_status::TransformResult,
};

pub struct TemplateTransform {}

impl TemplateTransform {
    pub fn to_dd_tpl_expr(
        expr: &mut Expr,
        csi_methods: &CsiMethods,
        ident_provider: &mut dyn IdentProvider,
    ) -> TransformResult<Expr> {
        if let Expr::Tpl(tpl) = expr {
            let mut assignations = Vec::new();
            let mut arguments = Vec::new();

            tpl.exprs.iter_mut().for_each(|tpl_expr| {
                let span = tpl_expr.span();

                DefaultOperandHandler::replace_expressions_in_expr(
                    tpl_expr,
                    IdentMode::Replace,
                    &mut assignations,
                    &mut arguments,
                    &span,
                    ident_provider,
                    IdentKind::Expr,
                    ExpandArrays::No,
                )
            });

            let dd_expr = get_dd_paren_expr(
                expr,
                &arguments,
                &mut assignations,
                &csi_methods.get_dd_tpl_operator_name(),
                &expr.span(),
            );

            return TransformResult::modified(dd_expr);
        }

        TransformResult::not_modified()
    }
}
