/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use std::fmt::{self, Debug, Display};

use crate::{rewriter::Config, telemetry::IastTelemetry};

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Status {
    Modified,
    NotModified,
    Cancelled,
}

impl Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Debug)]
pub struct TransformStatus {
    pub status: Status,
    pub msg: Option<String>,
    pub telemetry: IastTelemetry,
}

impl TransformStatus {
    pub fn not_modified(config: &Config) -> TransformStatus {
        TransformStatus {
            status: Status::NotModified,
            msg: None,
            telemetry: IastTelemetry::new(config),
        }
    }
}

pub struct TransformResult<T> {
    pub expr: Option<T>,
    pub status: Status,
    pub tag: Option<String>,
}

impl<T> TransformResult<T> {
    pub fn not_modified() -> TransformResult<T> {
        TransformResult {
            expr: None,
            tag: None,
            status: Status::NotModified,
        }
    }

    pub fn modified(expr: T) -> TransformResult<T> {
        TransformResult {
            expr: Some(expr),
            tag: None,
            status: Status::Modified,
        }
    }

    pub fn modified_with_tag(expr: T, tag: String) -> TransformResult<T> {
        TransformResult {
            expr: Some(expr),
            tag: Some(tag),
            status: Status::Modified,
        }
    }

    pub fn is_modified(&self) -> bool {
        self.status == Status::Modified
    }
}
