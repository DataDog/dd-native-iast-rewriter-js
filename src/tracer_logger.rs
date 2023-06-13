/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use log::{LevelFilter, Log, Metadata, Record};
use std::str::FromStr;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

type LoggerFn<'a> = &'a (dyn Fn(&str, String) + Send + Sync);

#[wasm_bindgen(module = "/tracer_logger.js")]
extern "C" {
    #[wasm_bindgen (js_name = log, catch)]
    pub fn log(level: &JsValue, msg: &JsValue) -> anyhow::Result<JsValue, JsValue>;

    #[wasm_bindgen (js_name = setLogger, catch)]
    pub fn setLogger(logger: &JsValue) -> anyhow::Result<JsValue, JsValue>;
}

pub struct TracerLogger<'a> {
    pub logger: LoggerFn<'a>,
    pub level_filter: LevelFilter,
}

impl<'a> TracerLogger<'a> {
    pub fn new(level: &str, logger: LoggerFn<'a>) -> Self {
        TracerLogger {
            logger,
            level_filter: LevelFilter::from_str(level).unwrap_or(LevelFilter::Off),
        }
    }

    pub fn with_level(level: &str) -> Self {
        Self::new(level, &|level, msg| {
            log(&level.into(), &msg.into()).ok();
        })
    }

    pub fn default() -> Self {
        Self::with_level("ERROR")
    }
}

impl Log for TracerLogger<'_> {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level().to_level_filter() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            (self.logger)(record.level().as_str(), format!("{}", record.args()));
        }
    }

    fn flush(&self) {}
}

pub fn set_logger(logger: &JsValue, level: &str) -> anyhow::Result<JsValue, JsValue> {
    log::set_max_level(LevelFilter::from_str(level).unwrap_or(log::max_level()));
    setLogger(logger)
}

// NOTE: log::set_boxed_logger should be called only once. Maybe in a function when wasm module is initialized. See #[wasm_bindgen(start)]
pub fn init() {
    let tracer_logger = TracerLogger::default();
    log::set_boxed_logger(Box::new(tracer_logger))
        .map(|_| log::set_max_level(LevelFilter::Off))
        .ok();
}
