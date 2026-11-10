//! The watchman: reads every probe on a steady beat, asks the rules what they think, and keeps
//! the short memory (history, baseline, first-seen times) that makes the answers useful.
//!
//! * [`readings`]  – all probe readings of one moment
//! * [`history`]   – the last hour, for trends
//! * [`baseline`]  – what the machine looked like at start
//! * [`ledger`]    – when each finding first appeared
//! * [`snapshot`]  – what the window shows
//! * [`preflight`] – the full check on demand

pub mod baseline;
pub mod history;
pub mod ledger;
pub mod preflight;
pub mod readings;
pub mod snapshot;

use crate::clock::{self, Millis};
use crate::probes::{
    BatteryProbe, DeviceProbe, FingerprintProbe, MachineProbe, NetworkProbe, PostureProbe, ProcessProbe,
    ProcessorProbe, Scheduled, SensorProbe, SoftwareProbe, StorageProbe,
};
use crate::rules::{Context, Evaluation, RuleBook};
use crate::{Finding, Severity};
use baseline::Baseline;
use history::{History, Sample};
use ledger::Ledger;
use preflight::{ConnectionChecks, PreflightReport};
use readings::Readings;
use snapshot::Snapshot;
use std::time::Duration;

/// What one beat of the watchman produced.
pub struct Tick {
    /// The state to show.
    pub snapshot: Snapshot,
    /// Findings that appeared on this beat and deserve a notification.
    pub alerts: Vec<Finding>,
}

/// Every probe, each read at its own pace.
struct Probes {
    machine: Scheduled<MachineProbe>,
    processor: Scheduled<ProcessorProbe>,
    storage: Scheduled<StorageProbe>,
    sensors: Scheduled<SensorProbe>,
    battery: Scheduled<BatteryProbe>,
    devices: Scheduled<DeviceProbe>,
    network: Scheduled<NetworkProbe>,
    processes: Scheduled<ProcessProbe>,
    software: Scheduled<SoftwareProbe>,
    posture: Scheduled<PostureProbe>,
    fingerprints: Scheduled<FingerprintProbe>,
}

impl Probes {
    /// Prepares every probe.
    fn new() -> Self {
        Self {
            machine: Scheduled::new(MachineProbe),
            processor: Scheduled::new(ProcessorProbe::new()),
            storage: Scheduled::new(StorageProbe::new()),
            sensors: Scheduled::new(SensorProbe::new()),
            battery: Scheduled::new(BatteryProbe::new()),
            devices: Scheduled::new(DeviceProbe),
            network: Scheduled::new(NetworkProbe::new()),
            processes: Scheduled::new(ProcessProbe::new()),
            software: Scheduled::new(SoftwareProbe),
            posture: Scheduled::new(PostureProbe),
            fingerprints: Scheduled::new(FingerprintProbe),
        }
    }

    /// Reads every probe that is due, or all of them when `force` is set.
    fn read(&mut self, force: bool) -> Readings {
        Readings {
            machine: self.machine.reading(force),
            processor: self.processor.reading(force),
            disks: self.storage.reading(force),
            sensors: self.sensors.reading(force),
            battery: self.battery.reading(force),
            devices: self.devices.reading(force),
            network: self.network.reading(force),
            performance: self.processes.reading(force),
            software: self.software.reading(force),
            posture: self.posture.reading(force),
            fingerprints: self.fingerprints.reading(force),
        }
    }
}

/// Watches the machine. Create one, call [`Watchman::tick`] every [`Watchman::BEAT`].
pub struct Watchman {
    probes: Probes,
    rules: RuleBook,
    history: History,
    baseline: Option<Baseline>,
    ledger: Ledger,
    started_at: Millis,
}

impl Watchman {
    /// How often the watchman looks.
    pub const BEAT: Duration = Duration::from_secs(2);

    /// How much history is kept: one hour of beats.
    const HISTORY: usize = 3600 / 2;

    /// Prepares the watchman. Nothing is read until the first tick.
    pub fn new() -> Self {
        Self {
            probes: Probes::new(),
            rules: RuleBook::standard(),
            history: History::new(Self::HISTORY),
            baseline: None,
            ledger: Ledger::new(),
            started_at: clock::now(),
        }
    }

    /// One beat: read what is due, judge it, remember it.
    pub fn tick(&mut self) -> Tick {
        let now = clock::now();
        let first = self.baseline.is_none();
        let readings = self.probes.read(false);
        self.history.push(Sample::from_readings(now, &readings));
        let mut evaluation = self.evaluate(now, &readings);
        let mut alerts = self.ledger.stamp(now, &mut evaluation.findings);
        // At start-up only urgent problems interrupt; the rest is waiting in the window.
        let threshold = if first { Severity::Critical } else { Severity::Warning };
        alerts.retain(|f| f.severity >= threshold);
        Tick { snapshot: Snapshot::new(now, self.started_at, evaluation, readings), alerts }
    }

    /// The last hour of samples, oldest first.
    pub fn history(&self) -> Vec<Sample> {
        self.history.samples()
    }

    /// A Preflight run in one go. Reads every probe fresh, so it takes a few seconds.
    pub fn preflight(&mut self) -> PreflightReport {
        let started = clock::now();
        let connection = ConnectionChecks::run();
        let evaluation = self.evaluate_fresh();
        PreflightReport::new(started, evaluation, connection)
    }

    /// Reads every probe fresh and runs every rule, without touching the ledger. Used by
    /// Preflight when the connection checks run elsewhere.
    pub fn evaluate_fresh(&mut self) -> Evaluation {
        let now = clock::now();
        let readings = self.probes.read(true);
        self.evaluate(now, &readings)
    }

    /// Runs every rule over the readings, taking the baseline on the very first look.
    fn evaluate(&mut self, now: Millis, readings: &Readings) -> Evaluation {
        let baseline = self.baseline.get_or_insert_with(|| Baseline::capture(now, readings));
        baseline.adopt_missing_fingerprints(readings);
        let context = Context { now, readings, history: &self.history, baseline };
        self.rules.evaluate(&context)
    }
}

impl Default for Watchman {
    /// Same as [`Watchman::new`].
    fn default() -> Self {
        Self::new()
    }
}
