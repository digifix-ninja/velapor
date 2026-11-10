//! What the machine looked like when Velapor started, so that changes stand out.
//!
//! The baseline lives in memory only. Velapor stores nothing on disk, so "new" always means
//! "new since Velapor was opened".

use super::readings::Readings;
use crate::clock::Millis;
use std::collections::{BTreeMap, BTreeSet};

/// The first look at the parts of the machine that should not change on their own.
#[derive(Debug, Clone)]
pub struct Baseline {
    /// When the first look was taken.
    pub taken_at: Millis,
    startup_items: BTreeSet<String>,
    listening_ports: BTreeSet<String>,
    devices: BTreeSet<String>,
    fingerprints: BTreeMap<String, String>,
}

impl Baseline {
    /// Remembers the current state as the reference point.
    pub fn capture(taken_at: Millis, readings: &Readings) -> Self {
        Self {
            taken_at,
            startup_items: readings.software.startup_items.iter().map(|i| i.key()).collect(),
            listening_ports: readings.network.listening.iter().map(|p| p.key()).collect(),
            devices: readings.devices.iter().map(|d| d.key.clone()).collect(),
            fingerprints: readings.fingerprints.iter().map(|f| (f.key.clone(), f.hash.clone())).collect(),
        }
    }

    /// Fingerprints taken later (for example the DNS servers, which only answer once the
    /// network is up) join the baseline the first time they appear.
    pub fn adopt_missing_fingerprints(&mut self, readings: &Readings) {
        for fingerprint in &readings.fingerprints {
            self.fingerprints.entry(fingerprint.key.clone()).or_insert_with(|| fingerprint.hash.clone());
        }
    }

    /// Whether a startup item was already there at the start.
    pub fn knows_startup_item(&self, key: &str) -> bool {
        self.startup_items.contains(key)
    }

    /// Whether a listening port was already open at the start.
    pub fn knows_listening_port(&self, key: &str) -> bool {
        self.listening_ports.contains(key)
    }

    /// Whether a device was already attached at the start.
    pub fn knows_device(&self, key: &str) -> bool {
        self.devices.contains(key)
    }

    /// The fingerprint a watched file or setting had at the start, if it was seen then.
    pub fn fingerprint(&self, key: &str) -> Option<&str> {
        self.fingerprints.get(key).map(String::as_str)
    }
}
