//! Probes: small collectors that each read one part of the machine.
//!
//! Every probe implements [`Probe`]. A probe only reads; when something is not available on a
//! platform it says so (an empty list, `None`, or "unknown") instead of guessing.
//!
//! [`Scheduled`] wraps a probe and reads it only as often as its [`Probe::interval`] asks, so
//! slow questions (the machine's model, security settings) are not asked every two seconds.

mod battery;
mod devices;
mod fingerprints;
mod machine;
mod network;
mod posture;
mod processes;
mod processor;
mod sensors;
mod software;
mod storage;

pub use battery::{BatteryProbe, BatteryReading};
pub use devices::{AttachedDevice, DeviceProbe};
pub use fingerprints::{Fingerprint, FingerprintProbe};
pub use machine::{MachineProbe, MachineReport};
pub use network::{InterfaceReading, ListeningPort, NetworkProbe, NetworkReport, ProcessConnections};
pub use posture::{PostureCheck, PostureProbe, PostureState};
pub use processes::{PerformanceReport, ProcessProbe, ProcessReading};
pub use processor::{CpuReading, MemoryReading, ProcessorProbe, ProcessorReport};
pub use sensors::{SensorProbe, SensorReading};
pub use software::{SoftwareProbe, SoftwareReport, StartupItem};
pub use storage::{DiskReading, StorageProbe};

use std::time::{Duration, Instant};

/// A collector that reads one part of the machine and never changes it.
pub trait Probe {
    /// What the probe hands back.
    type Report: Clone;

    /// How often a fresh reading is worth taking.
    fn interval(&self) -> Duration;

    /// Takes one reading.
    fn read(&mut self) -> Self::Report;
}

/// A probe with its last reading, read again only when its interval has passed.
pub struct Scheduled<P: Probe> {
    probe: P,
    latest: Option<P::Report>,
    read_at: Option<Instant>,
}

impl<P: Probe> Scheduled<P> {
    /// Wraps a probe that has not been read yet.
    pub fn new(probe: P) -> Self {
        Self { probe, latest: None, read_at: None }
    }

    /// The latest reading, taken now if it is due (or if `force` asks for a fresh one).
    pub fn reading(&mut self, force: bool) -> P::Report {
        let due = self.read_at.is_none_or(|at| at.elapsed() >= self.probe.interval());
        if force || due || self.latest.is_none() {
            self.latest = Some(self.probe.read());
            self.read_at = Some(Instant::now());
        }
        self.latest.clone().expect("a reading was just taken")
    }
}
