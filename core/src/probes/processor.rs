//! Processor load and memory.

use super::Probe;
use serde::Serialize;
use std::time::Duration;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

/// What the processor is doing right now.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuReading {
    /// Average load over all cores, 0–100.
    pub usage: f32,
    /// Logical cores.
    pub cores: usize,
    /// Physical cores, when the system says.
    pub physical_cores: Option<usize>,
    /// Current frequency of the first core, in MHz.
    pub frequency_mhz: u64,
    /// The processor's name, e.g. "Apple M2 Max".
    pub brand: String,
    /// Load average over 1, 5 and 15 minutes (macOS and Linux).
    pub load_average: Option<[f64; 3]>,
}

/// How memory is used.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryReading {
    /// Installed memory in bytes.
    pub total: u64,
    /// Memory in use, in bytes.
    pub used: u64,
    /// Memory still available to programs, in bytes.
    pub available: u64,
    /// Swap space in bytes.
    pub swap_total: u64,
    /// Swap in use, in bytes.
    pub swap_used: u64,
}

impl MemoryReading {
    /// Share of memory in use, 0–100.
    pub fn used_percent(&self) -> f64 {
        share(self.used, self.total)
    }

    /// Share of swap in use, 0–100.
    pub fn swap_percent(&self) -> f64 {
        share(self.swap_used, self.swap_total)
    }
}

/// Processor and memory together: both come from the same system query.
#[derive(Debug, Clone, Serialize)]
pub struct ProcessorReport {
    /// The processor.
    pub cpu: CpuReading,
    /// The memory.
    pub memory: MemoryReading,
}

impl ProcessorReport {
    /// What is reported when the system gives no answer at all.
    fn unknown() -> Self {
        Self {
            cpu: CpuReading {
                usage: 0.0,
                cores: 0,
                physical_cores: None,
                frequency_mhz: 0,
                brand: String::new(),
                load_average: None,
            },
            memory: MemoryReading { total: 0, used: 0, available: 0, swap_total: 0, swap_used: 0 },
        }
    }
}

/// Reads processor load and memory use.
pub struct ProcessorProbe {
    /// `None` only if the system refused to answer at all.
    system: Option<System>,
}

impl ProcessorProbe {
    /// Prepares the probe. The first load reading needs a moment to settle.
    pub fn new() -> Self {
        let kind =
            RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()).with_memory(MemoryRefreshKind::everything());
        Self { system: System::new_with_specifics(kind).ok() }
    }
}

impl Default for ProcessorProbe {
    /// Same as [`ProcessorProbe::new`].
    fn default() -> Self {
        Self::new()
    }
}

impl Probe for ProcessorProbe {
    type Report = ProcessorReport;

    /// Load changes quickly: read it on every tick.
    fn interval(&self) -> Duration {
        Duration::ZERO
    }

    /// Reads load and memory.
    fn read(&mut self) -> ProcessorReport {
        let Some(system) = self.system.as_mut() else {
            return ProcessorReport::unknown();
        };
        system.refresh_cpu_usage();
        system.refresh_cpu_frequency();
        system.refresh_memory();
        let cpus = system.cpus();
        let load = System::load_average().ok().filter(|l| l.one > 0.0 || l.five > 0.0);
        ProcessorReport {
            cpu: CpuReading {
                usage: system.global_cpu_usage(),
                cores: cpus.len(),
                physical_cores: System::physical_core_count().ok(),
                frequency_mhz: cpus.first().map(|c| c.frequency()).unwrap_or(0),
                brand: cpus.first().map(|c| c.brand().trim().to_string()).unwrap_or_default(),
                load_average: load.map(|l| [l.one, l.five, l.fifteen]),
            },
            memory: MemoryReading {
                total: system.total_memory(),
                used: system.used_memory(),
                available: system.available_memory(),
                swap_total: system.total_swap(),
                swap_used: system.used_swap(),
            },
        }
    }
}

/// `part` as a share of `whole`, 0–100 (0 when there is no whole).
pub(crate) fn share(part: u64, whole: u64) -> f64 {
    if whole == 0 {
        0.0
    } else {
        part as f64 / whole as f64 * 100.0
    }
}
