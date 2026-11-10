//! Everything the probes saw on one tick, gathered in one place.

use crate::probes::{
    AttachedDevice, BatteryReading, DiskReading, Fingerprint, MachineReport, NetworkReport, PerformanceReport,
    PostureCheck, ProcessorReport, SensorReading, SoftwareReport,
};
use serde::Serialize;

/// The latest reading of every probe. Rules look at this; the window shows it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Readings {
    /// What the machine is: model, processor, memory, system.
    pub machine: MachineReport,
    /// Processor load and memory use.
    pub processor: ProcessorReport,
    /// Mounted drives.
    pub disks: Vec<DiskReading>,
    /// Temperature sensors.
    pub sensors: Vec<SensorReading>,
    /// The battery, if there is one.
    pub battery: Option<BatteryReading>,
    /// Everything attached: USB, Bluetooth, power adapter.
    pub devices: Vec<AttachedDevice>,
    /// Interfaces, traffic, open ports and connections.
    pub network: NetworkReport,
    /// Processes and disk activity.
    pub performance: PerformanceReport,
    /// The operating system and what starts with it.
    pub software: SoftwareReport,
    /// Security settings.
    pub posture: Vec<PostureCheck>,
    /// Fingerprints of files and settings that should stay still.
    pub fingerprints: Vec<Fingerprint>,
}

impl Readings {
    /// The warmest temperature sensor, if any sensor answers.
    pub fn hottest_sensor(&self) -> Option<&SensorReading> {
        self.sensors.iter().max_by(|a, b| a.celsius.total_cmp(&b.celsius))
    }
}
