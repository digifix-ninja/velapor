//! Shared data types used by every part of Velapor.
//!
//! Nothing in this crate touches the system. It only describes what Velapor
//! measures (samples) and what it wants to tell the user (findings).

use std::time::SystemTime;

/// How serious a finding is. Ordered from calm to urgent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// Everything looks fine.
    Clear,
    /// Worth knowing, nothing to do yet.
    Notice,
    /// Keep an eye on it.
    Warning,
    /// Look at this now.
    Critical,
}

impl Severity {
    /// A short, human label for the severity.
    pub fn label(self) -> &'static str {
        match self {
            Severity::Clear => "Clear",
            Severity::Notice => "Notice",
            Severity::Warning => "Warning",
            Severity::Critical => "Critical",
        }
    }
}

/// The areas of the system Velapor watches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Domain {
    /// CPU, GPU, memory, storage, sensors, battery.
    Hardware,
    /// Ports, connections, bandwidth, latency, DNS.
    Network,
    /// Services, drivers, startup items, logs, updates.
    System,
    /// Bottlenecks, hungry processes, trends.
    Performance,
    /// Changes to the parts of the system that should stay still.
    Integrity,
}

/// One measured value, such as a temperature or a load percentage.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A number with its unit, e.g. 61.5 "°C".
    Number { value: f64, unit: &'static str },
    /// A piece of text, e.g. a drive's model name.
    Text(String),
    /// A yes or no answer, e.g. "is FileVault on?".
    Flag(bool),
    /// The platform cannot provide this measurement.
    Unsupported,
}

/// A single named reading inside a sample.
#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    /// Stable key, e.g. "cpu.load.total".
    pub key: String,
    /// The measured value.
    pub value: Value,
}

/// Everything one probe measured in one run.
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    /// The probe that took this sample.
    pub probe_id: &'static str,
    /// When the sample was taken.
    pub taken_at: SystemTime,
    /// The individual readings.
    pub readings: Vec<Reading>,
}

impl Sample {
    /// Creates an empty sample for the given probe, stamped with the current time.
    pub fn new(probe_id: &'static str) -> Self {
        Self { probe_id, taken_at: SystemTime::now(), readings: Vec::new() }
    }

    /// Adds a reading and returns the sample, so readings can be chained.
    pub fn with(mut self, key: impl Into<String>, value: Value) -> Self {
        self.readings.push(Reading { key: key.into(), value });
        self
    }
}

/// A measurement that backs up a finding.
#[derive(Debug, Clone, PartialEq)]
pub struct Evidence {
    /// What was measured, in plain words.
    pub description: String,
    /// The reading it is based on.
    pub reading: Reading,
}

/// Something Velapor wants to tell the user. Every finding carries its evidence.
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    /// Stable identifier, e.g. "disk.smart.reallocated_sectors".
    pub id: String,
    /// The area this finding belongs to.
    pub domain: Domain,
    /// How serious it is.
    pub severity: Severity,
    /// A one-line headline.
    pub title: String,
    /// What this means, in plain language.
    pub explanation: String,
    /// Why it is probably happening.
    pub likely_cause: String,
    /// What the user can do about it.
    pub recommendation: String,
    /// The measurements behind the finding.
    pub evidence: Vec<Evidence>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severities_are_ordered_from_calm_to_urgent() {
        assert!(Severity::Clear < Severity::Notice);
        assert!(Severity::Warning < Severity::Critical);
    }

    #[test]
    fn sample_collects_readings() {
        let sample = Sample::new("test").with("answer", Value::Number { value: 42.0, unit: "" });
        assert_eq!(sample.readings.len(), 1);
    }
}
