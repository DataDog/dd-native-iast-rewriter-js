#![deny(clippy::all)]
/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
mod rewriter;
mod telemetry;
// mod tracer_logger;
mod transform;
mod util;
mod visitor;

#[cfg(test)]
mod tests;

#[cfg(not(feature = "napi"))]
mod lib_wasm;

#[macro_use]
#[cfg(feature = "napi")]
extern crate napi_derive;

#[cfg(feature = "napi")]
mod lib_napi;
