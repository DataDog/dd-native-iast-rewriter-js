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
    pub msg: String,
}

impl TransformStatus {
    pub fn not_modified() -> TransformStatus {
        TransformStatus {
            status: Status::NotModified,
            msg: String::from(""),
        }
    }

    pub fn modified() -> TransformStatus {
        TransformStatus {
            status: Status::Modified,
            msg: String::from(""),
        }
    }
}
