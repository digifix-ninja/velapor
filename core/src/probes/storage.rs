//! Drives and volumes: how full they are and how busy.

use super::processor::share;
use super::Probe;
use serde::Serialize;
use std::time::{Duration, Instant};
use sysinfo::{DiskKind, DiskRefreshKind, Disks};

/// One mounted volume.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskReading {
    /// The volume's name, e.g. "Macintosh HD" or "/dev/nvme0n1p2".
    pub name: String,
    /// Where it is mounted, e.g. "/" or "C:\".
    pub mount_point: String,
    /// The file system, e.g. "apfs", "ext4", "NTFS".
    pub file_system: String,
    /// "SSD", "HDD" or "Drive" when the system doesn't say.
    pub kind: String,
    /// Size in bytes.
    pub total: u64,
    /// Free space in bytes.
    pub available: u64,
    /// A drive that can be unplugged (USB stick, external disk).
    pub removable: bool,
    /// Mounted read-only (e.g. the macOS system volume).
    pub read_only: bool,
    /// Bytes read per second since the last reading.
    pub read_per_sec: u64,
    /// Bytes written per second since the last reading.
    pub written_per_sec: u64,
}

impl DiskReading {
    /// Share of space in use, 0–100.
    pub fn used_percent(&self) -> f64 {
        share(self.total.saturating_sub(self.available), self.total)
    }
}

/// Reads every mounted volume.
pub struct StorageProbe {
    disks: Option<Disks>,
    last_read: Instant,
}

impl StorageProbe {
    /// Finds the volumes.
    pub fn new() -> Self {
        Self { disks: Disks::new_with_refreshed_list().ok(), last_read: Instant::now() }
    }

    /// The volumes worth showing: real storage, not the many small system mounts.
    fn worth_showing(reading: &DiskReading) -> bool {
        let system_mounts =
            ["/boot", "/snap/", "/System/Volumes/", "/private/var/vm", "/run/", "/dev", "/var/lib/docker"];
        let hidden = system_mounts
            .iter()
            .any(|m| reading.mount_point.starts_with(m) && reading.mount_point != "/System/Volumes/Data");
        reading.total > 1_000_000_000 && !hidden
    }
}

impl Default for StorageProbe {
    /// Same as [`StorageProbe::new`].
    fn default() -> Self {
        Self::new()
    }
}

impl Probe for StorageProbe {
    type Report = Vec<DiskReading>;

    /// Space changes slowly, activity quickly: every few seconds is a good middle.
    fn interval(&self) -> Duration {
        Duration::from_secs(4)
    }

    /// Reads size, free space and activity of each volume.
    fn read(&mut self) -> Vec<DiskReading> {
        let Some(disks) = self.disks.as_mut() else { return Vec::new() };
        disks.refresh_specifics(true, DiskRefreshKind::everything());
        let seconds = self.last_read.elapsed().as_secs_f64().max(0.5);
        self.last_read = Instant::now();
        let mut readings: Vec<DiskReading> = disks
            .iter()
            .map(|disk| {
                let usage = disk.usage();
                DiskReading {
                    name: disk.name().to_string_lossy().into_owned(),
                    mount_point: disk.mount_point().to_string_lossy().into_owned(),
                    file_system: disk.file_system().to_string_lossy().into_owned(),
                    kind: match disk.kind() {
                        DiskKind::SSD => "SSD",
                        DiskKind::HDD => "HDD",
                        _ => "Drive",
                    }
                    .to_string(),
                    total: disk.total_space(),
                    available: disk.available_space(),
                    removable: disk.is_removable(),
                    read_only: disk.is_read_only(),
                    read_per_sec: (usage.read_bytes as f64 / seconds) as u64,
                    written_per_sec: (usage.written_bytes as f64 / seconds) as u64,
                }
            })
            .filter(Self::worth_showing)
            .collect();
        // On macOS the same space shows up as "/" and "/System/Volumes/Data": keep one.
        readings.sort_by_key(|d| d.mount_point.len());
        readings
            .dedup_by(|b, a| a.name == b.name && a.total == b.total && a.available.abs_diff(b.available) < 50_000_000);
        readings
    }
}
