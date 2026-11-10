//! Running programs: which ones use the most processor time, memory and disk.

use super::Probe;
use serde::Serialize;
use std::time::{Duration, Instant};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// One running program.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessReading {
    /// Process id.
    pub pid: u32,
    /// The program's name.
    pub name: String,
    /// Share of the whole processor (all cores together), 0–100.
    pub cpu: f32,
    /// Memory in use, in bytes.
    pub memory: u64,
    /// Bytes read from disk per second.
    pub read_per_sec: u64,
    /// Bytes written to disk per second.
    pub written_per_sec: u64,
    /// How long it has been running, in seconds.
    pub run_time: u64,
}

/// What the performance probe sees.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceReport {
    /// Running processes.
    pub process_count: usize,
    /// The programs using the most processor time.
    pub by_cpu: Vec<ProcessReading>,
    /// The programs using the most memory.
    pub by_memory: Vec<ProcessReading>,
    /// Bytes read from disk per second, all programs together.
    pub disk_read_per_sec: u64,
    /// Bytes written to disk per second, all programs together.
    pub disk_written_per_sec: u64,
}

/// Reads the running programs.
pub struct ProcessProbe {
    system: Option<System>,
    last_read: Instant,
}

impl ProcessProbe {
    /// How many programs each list shows.
    const LIST_LENGTH: usize = 8;

    /// Prepares the probe.
    pub fn new() -> Self {
        Self { system: System::new().ok(), last_read: Instant::now() }
    }
}

impl Default for ProcessProbe {
    /// Same as [`ProcessProbe::new`].
    fn default() -> Self {
        Self::new()
    }
}

impl Probe for ProcessProbe {
    type Report = PerformanceReport;

    /// Programs change quickly; every tick.
    fn interval(&self) -> Duration {
        Duration::ZERO
    }

    /// Reads every running program and keeps the busiest.
    fn read(&mut self) -> PerformanceReport {
        let Some(system) = self.system.as_mut() else { return PerformanceReport::default() };
        system.refresh_cpu_usage();
        let kind =
            ProcessRefreshKind::nothing().with_cpu().with_memory().with_disk_usage().with_exe(UpdateKind::OnlyIfNotSet);
        system.refresh_processes_specifics(ProcessesToUpdate::All, true, kind);
        let seconds = self.last_read.elapsed().as_secs_f64().max(0.5);
        self.last_read = Instant::now();
        let cores = system.cpus().len().max(1) as f32;

        let all: Vec<ProcessReading> = system
            .processes()
            .iter()
            .filter(|(_, p)| p.thread_kind().is_none())
            .map(|(pid, p)| {
                let disk = p.disk_usage();
                ProcessReading {
                    pid: pid.as_u32(),
                    name: p.name().to_string_lossy().into_owned(),
                    cpu: p.cpu_usage() / cores,
                    memory: p.memory(),
                    read_per_sec: (disk.read_bytes as f64 / seconds) as u64,
                    written_per_sec: (disk.written_bytes as f64 / seconds) as u64,
                    run_time: p.run_time(),
                }
            })
            .collect();

        let mut by_cpu = all.clone();
        by_cpu.sort_by(|a, b| b.cpu.total_cmp(&a.cpu));
        by_cpu.truncate(Self::LIST_LENGTH);
        let mut by_memory = all.clone();
        by_memory.sort_by_key(|p| std::cmp::Reverse(p.memory));
        by_memory.truncate(Self::LIST_LENGTH);

        PerformanceReport {
            process_count: all.len(),
            disk_read_per_sec: all.iter().map(|p| p.read_per_sec).sum(),
            disk_written_per_sec: all.iter().map(|p| p.written_per_sec).sum(),
            by_cpu,
            by_memory,
        }
    }
}
