//! History for Velapor, owned by Sentry.
//!
//! Planned retention: raw samples for 24 hours, 1-minute averages for 7 days,
//! 1-hour averages for a year. SQLite arrives in phase 1; this crate only
//! fixes the shape of the API for now.

use std::time::Duration;

use velapor_core::Sample;

/// How long each resolution of history is kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Retention {
    /// Every sample as it was taken.
    pub raw: Duration,
    /// One-minute averages.
    pub minute: Duration,
    /// One-hour averages.
    pub hour: Duration,
}

impl Default for Retention {
    /// The retention described in ARCHITECTURE.md.
    fn default() -> Self {
        const DAY: u64 = 24 * 60 * 60;
        Self {
            raw: Duration::from_secs(DAY),
            minute: Duration::from_secs(7 * DAY),
            hour: Duration::from_secs(365 * DAY),
        }
    }
}

/// Somewhere samples can be kept and read back.
pub trait Store {
    /// Saves one sample.
    fn record(&mut self, sample: &Sample);
    /// Returns the most recent sample of a probe, if there is one.
    fn latest(&self, probe_id: &str) -> Option<&Sample>;
}

/// A store that only lives in memory. Useful for tests and the first end-to-end run.
#[derive(Default)]
pub struct MemoryStore {
    samples: Vec<Sample>,
}

impl Store for MemoryStore {
    /// Keeps the sample in memory.
    fn record(&mut self, sample: &Sample) {
        self.samples.push(sample.clone());
    }

    /// Finds the newest sample of the given probe.
    fn latest(&self, probe_id: &str) -> Option<&Sample> {
        self.samples.iter().rev().find(|s| s.probe_id == probe_id)
    }
}
