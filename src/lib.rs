#![deny(clippy::all)]
/**
* Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
* This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
**/
mod rewriter;
mod transform;
mod util;
mod visitor;

#[cfg(test)]
mod tests;

#[cfg(feature = "wasm")]
mod lib_wasm;

#[macro_use]
#[cfg(not(feature = "wasm"))]
extern crate napi_derive;

#[cfg(not(feature = "wasm"))]
mod lib_napi;
