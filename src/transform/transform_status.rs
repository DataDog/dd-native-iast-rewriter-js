use crate::{rewriter::Config, telemetry::IastTelemetry};

/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/

#[derive(PartialEq, Eq)]
pub enum Status {
    Modified,
    NotModified,
    Cancelled,
}

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
    pub expr: T,
    pub status: Status,
}

impl<T> TransformResult<T> {
    pub fn not_modified(expr: T) -> TransformResult<T> {
        TransformResult {
            expr,
            status: Status::NotModified,
        }
    }

    pub fn modified(expr: T) -> TransformResult<T> {
        TransformResult {
            expr,
            status: Status::Modified,
        }
    }

    pub fn is_modified(&self) -> bool {
        self.status == Status::Modified
    }
}
