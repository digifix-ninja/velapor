//! Velapor's probes: small collectors that each answer one question about the system.
//!
//! Each probe has a backend per operating system. When a measurement is not
//! available on a platform, the probe says so honestly instead of pretending.

use std::fmt;
use std::time::Duration;

use velapor_core::{Sample, Value};

/// Why a probe could not take a measurement.
#[derive(Debug)]
pub enum ProbeError {
    /// This measurement does not exist on the current platform.
    Unsupported,
    /// The probe lacks the rights it needs (it should run inside Sentry).
    PermissionDenied,
    /// Anything else, with a short explanation.
    Failed(String),
}

impl fmt::Display for ProbeError {
    /// Formats the error as a short sentence.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProbeError::Unsupported => write!(f, "not supported on this platform"),
            ProbeError::PermissionDenied => write!(f, "permission denied"),
            ProbeError::Failed(reason) => write!(f, "probe failed: {reason}"),
        }
    }
}

impl std::error::Error for ProbeError {}

/// A collector that measures one aspect of the system.
pub trait Probe {
    /// Stable identifier, e.g. "hardware.disk.smart".
    fn id(&self) -> &'static str;
    /// How often this probe should run by default.
    fn interval(&self) -> Duration;
    /// Takes one measurement.
    fn sample(&mut self) -> Result<Sample, ProbeError>;
}

/// The first probe end to end: how long the machine has been running.
/// It needs no privileges, which makes it a good first passenger for the IPC channel.
pub struct UptimeProbe;

impl Probe for UptimeProbe {
    /// Returns the probe's identifier.
    fn id(&self) -> &'static str {
        "system.uptime"
    }

    /// Uptime changes slowly, once a minute is plenty.
    fn interval(&self) -> Duration {
        Duration::from_secs(60)
    }

    /// Reads the uptime in seconds, or reports it as unsupported.
    fn sample(&mut self) -> Result<Sample, ProbeError> {
        let value = match uptime_seconds() {
            Some(seconds) => Value::Number { value: seconds, unit: "s" },
            None => Value::Unsupported,
        };
        Ok(Sample::new(self.id()).with("system.uptime", value))
    }
}

/// Linux: reads the first number from /proc/uptime.
#[cfg(target_os = "linux")]
fn uptime_seconds() -> Option<f64> {
    let text = std::fs::read_to_string("/proc/uptime").ok()?;
    text.split_whitespace().next()?.parse().ok()
}

/// macOS and Windows backends arrive in phase 1.
#[cfg(not(target_os = "linux"))]
fn uptime_seconds() -> Option<f64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uptime_probe_always_returns_a_sample() {
        let sample = UptimeProbe.sample().expect("uptime probe should not fail");
        assert_eq!(sample.probe_id, "system.uptime");
        assert_eq!(sample.readings.len(), 1);
    }
}
