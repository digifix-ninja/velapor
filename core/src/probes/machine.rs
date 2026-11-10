//! What the machine is: model, processor, memory, graphics and operating system.
//! Read once at start and then only now and then: none of it changes while Velapor runs.

use super::Probe;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use crate::system_query::SystemQuery;
use serde::Serialize;
use std::time::Duration;
use sysinfo::{CpuRefreshKind, Gpus, MemoryRefreshKind, RefreshKind, System};

/// The machine's specifications.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MachineReport {
    /// The model as people know it, e.g. "MacBook Pro" or "ThinkPad X1 Carbon Gen 11".
    pub model: Option<String>,
    /// The maker, e.g. "Apple" or "LENOVO".
    pub maker: Option<String>,
    /// The model identifier, e.g. "Mac14,6".
    pub model_id: Option<String>,
    /// The processor, e.g. "Apple M2 Max".
    pub processor: String,
    /// Logical cores.
    pub cores: usize,
    /// Physical cores, when known.
    pub physical_cores: Option<usize>,
    /// Installed memory in bytes.
    pub memory: u64,
    /// Graphics processors.
    pub graphics: Vec<String>,
    /// Operating system with version, e.g. "macOS 15.4 Sequoia".
    pub system: String,
    /// The kernel version.
    pub kernel: String,
    /// The computer's network name.
    pub host_name: String,
    /// Processor architecture, e.g. "arm64" or "x86_64".
    pub architecture: String,
}

/// Reads the machine's specifications.
pub struct MachineProbe;

impl MachineProbe {
    /// Model, maker and model id, asked the way each system answers best.
    #[cfg(target_os = "macos")]
    fn model() -> (Option<String>, Option<String>, Option<String>) {
        let model_id = SystemQuery::run("sysctl", &["-n", "hw.model"]).map(|s| s.trim().to_string());
        let name = SystemQuery::run("system_profiler", &["SPHardwareDataType"]).and_then(|text| {
            text.lines().find_map(|line| line.trim().strip_prefix("Model Name:").map(|v| v.trim().to_string()))
        });
        (name, Some("Apple".into()), model_id)
    }

    /// Model, maker and model id from the firmware tables Linux exposes to every user.
    #[cfg(target_os = "linux")]
    fn model() -> (Option<String>, Option<String>, Option<String>) {
        let read = |name: &str| {
            std::fs::read_to_string(format!("/sys/class/dmi/id/{name}"))
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && !s.contains("To Be Filled") && s != "System Product Name")
        };
        let version = read("product_version").filter(|v| v.chars().any(|c| c.is_alphabetic()) && v.len() > 2);
        let name = read("product_name").map(|n| match &version {
            Some(v) if !n.contains(v.as_str()) => format!("{n} {v}"),
            _ => n,
        });
        (name, read("sys_vendor"), read("product_name"))
    }

    /// Model and maker from Windows' own inventory.
    #[cfg(target_os = "windows")]
    fn model() -> (Option<String>, Option<String>, Option<String>) {
        let answer =
            SystemQuery::powershell("$c = Get-CimInstance Win32_ComputerSystem; \"$($c.Manufacturer)|$($c.Model)\"");
        let mut parts = answer.as_deref().unwrap_or("").splitn(2, '|').map(|s| s.trim().to_string());
        let maker = parts.next().filter(|s| !s.is_empty());
        let model = parts.next().filter(|s| !s.is_empty());
        (model.clone(), maker, model)
    }

    /// Other systems: nothing to ask.
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn model() -> (Option<String>, Option<String>, Option<String>) {
        (None, None, None)
    }
}

impl Probe for MachineProbe {
    type Report = MachineReport;

    /// The hardware doesn't change while Velapor runs; an hour is plenty.
    fn interval(&self) -> Duration {
        Duration::from_secs(3600)
    }

    /// Reads the specifications.
    fn read(&mut self) -> MachineReport {
        let kind =
            RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()).with_memory(MemoryRefreshKind::everything());
        let system = System::new_with_specifics(kind).ok();
        let (model, maker, model_id) = Self::model();
        let graphics = Gpus::new_with_refreshed_list()
            .map(|gpus| gpus.iter().filter_map(|g| g.model().map(str::to_string)).collect())
            .unwrap_or_default();
        MachineReport {
            model,
            maker,
            model_id,
            processor: system
                .as_ref()
                .and_then(|s| s.cpus().first().map(|c| c.brand().trim().to_string()))
                .unwrap_or_default(),
            cores: system.as_ref().map(|s| s.cpus().len()).unwrap_or(0),
            physical_cores: System::physical_core_count().ok(),
            memory: system.as_ref().map(|s| s.total_memory()).unwrap_or(0),
            graphics,
            system: System::long_os_version().unwrap_or_else(|_| std::env::consts::OS.to_string()),
            kernel: System::kernel_version().unwrap_or_default(),
            host_name: System::host_name().unwrap_or_default(),
            architecture: System::cpu_arch(),
        }
    }
}
