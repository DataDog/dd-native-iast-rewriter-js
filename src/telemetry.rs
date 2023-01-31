/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2022 Datadog, Inc.
 **/
use std::{collections::HashMap, fmt::Debug};

use crate::rewriter::Config;

#[derive(PartialEq, Eq, serde::Deserialize, serde::Serialize, Clone, Debug)]
pub enum TelemetryVerbosity {
    Off,
    Mandatory,
    Information,
    Debug,
}

pub trait Telemetry {
    fn inc(&mut self, optional_tag: Option<String>);

    fn get_instrumented_propagation(&self) -> u32;

    fn get_propagation_debug(&self) -> Option<HashMap<String, u32>>;
}

#[derive(Debug)]
pub enum IastTelemetry {
    Default(DefaultTelemetry),
    Debug(DebugTelemetry),
    NoOp(NoOpTelemetry),
}

impl IastTelemetry {
    pub fn new(config: &Config) -> IastTelemetry {
        match config.verbosity {
            TelemetryVerbosity::Off => IastTelemetry::NoOp(NoOpTelemetry {}),
            TelemetryVerbosity::Debug => IastTelemetry::Debug(DebugTelemetry::new()),
            _ => IastTelemetry::Default(DefaultTelemetry::new()),
        }
    }
}

impl Telemetry for IastTelemetry {
    fn inc(&mut self, tag: Option<String>) {
        match self {
            IastTelemetry::Default(t) => t.inc(tag),
            IastTelemetry::Debug(t) => t.inc(tag),
            IastTelemetry::NoOp(t) => t.inc(tag),
        }
    }

    fn get_instrumented_propagation(&self) -> u32 {
        match self {
            IastTelemetry::Default(t) => t.get_instrumented_propagation(),
            IastTelemetry::Debug(t) => t.get_instrumented_propagation(),
            IastTelemetry::NoOp(t) => t.get_instrumented_propagation(),
        }
    }

    fn get_propagation_debug(&self) -> Option<HashMap<String, u32>> {
        match self {
            IastTelemetry::Default(t) => t.get_propagation_debug(),
            IastTelemetry::Debug(t) => t.get_propagation_debug(),
            IastTelemetry::NoOp(t) => t.get_propagation_debug(),
        }
    }
}

#[derive(Debug)]
pub struct DefaultTelemetry {
    pub instrumented_propagation: u32,
}

impl DefaultTelemetry {
    pub fn new() -> Self {
        DefaultTelemetry {
            instrumented_propagation: 0,
        }
    }
}

impl Telemetry for DefaultTelemetry {
    fn inc(&mut self, _optional_tag: Option<String>) {
        self.instrumented_propagation += 1;
    }

    fn get_instrumented_propagation(&self) -> u32 {
        self.instrumented_propagation
    }

    fn get_propagation_debug(&self) -> Option<HashMap<String, u32>> {
        None
    }
}

#[derive(Debug)]
pub struct DebugTelemetry {
    pub instrumented_propagation: u32,
    pub propagation_debug: HashMap<String, u32>,
}

impl DebugTelemetry {
    fn new() -> Self {
        DebugTelemetry {
            instrumented_propagation: 0,
            propagation_debug: HashMap::new(),
        }
    }
}

impl Telemetry for DebugTelemetry {
    fn inc(&mut self, optional_tag: Option<String>) {
        self.instrumented_propagation += 1;

        if let Some(tag) = optional_tag {
            let value = self.propagation_debug.get(&tag);
            let mut end_value = 1;
            if let Some(counter) = value {
                end_value = counter + 1;
            }
            self.propagation_debug.insert(tag, end_value);
        }
    }

    fn get_instrumented_propagation(&self) -> u32 {
        self.instrumented_propagation
    }

    fn get_propagation_debug(&self) -> Option<HashMap<String, u32>> {
        Some(self.propagation_debug.clone())
    }
}

#[derive(Debug)]
pub struct NoOpTelemetry {}

impl Telemetry for NoOpTelemetry {
    fn inc(&mut self, _optional_tag: Option<String>) {}

    fn get_instrumented_propagation(&self) -> u32 {
        0
    }

    fn get_propagation_debug(&self) -> Option<HashMap<String, u32>> {
        None
    }
}
