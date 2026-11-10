//! The operating system and the programs that start with it.

use super::Probe;
#[allow(unused_imports)]
use crate::system_query::SystemQuery;
use serde::Serialize;
#[allow(unused_imports)]
use std::path::{Path, PathBuf};
use std::time::Duration;
use sysinfo::System;

/// Something that starts by itself: a launch agent, an autostart entry, a Run key…
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupItem {
    /// Its name, e.g. "net.syncthing" or "OneDrive".
    pub name: String,
    /// What kind it is, e.g. "Launch agent", "Autostart entry", "Run key".
    pub kind: String,
    /// Where it is registered (a folder or a registry key).
    pub location: String,
}

impl StartupItem {
    /// The identity of an item: its kind, location and name.
    pub fn key(&self) -> String {
        format!("{}|{}|{}", self.kind, self.location, self.name)
    }
}

/// What the software probe sees.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareReport {
    /// The system's name and version, e.g. "macOS 15.4 Sequoia".
    pub system: String,
    /// The version alone, e.g. "15.4".
    pub version: String,
    /// The kernel version.
    pub kernel: String,
    /// The computer's network name.
    pub host_name: String,
    /// Seconds since the machine started.
    pub uptime: u64,
    /// When the machine started, in seconds since 1970.
    pub boot_time: u64,
    /// Everything that starts by itself.
    pub startup_items: Vec<StartupItem>,
}

/// Reads the operating system and its startup items.
pub struct SoftwareProbe;

impl SoftwareProbe {
    /// The names of files in a folder that end with `extension`, as startup items.
    #[allow(dead_code)]
    fn folder_items(folder: &Path, extension: &str, kind: &str) -> Vec<StartupItem> {
        let Ok(entries) = std::fs::read_dir(folder) else { return Vec::new() };
        let mut items: Vec<StartupItem> = entries
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                let stem = name.strip_suffix(extension)?;
                Some(StartupItem { name: stem.to_string(), kind: kind.into(), location: folder.display().to_string() })
            })
            .collect();
        items.sort_by(|a, b| a.name.cmp(&b.name));
        items
    }

    /// The person's home folder.
    #[allow(dead_code)]
    fn home() -> Option<PathBuf> {
        std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from)
    }

    /// macOS: launch agents and daemons in the person's and the system's Library.
    #[cfg(target_os = "macos")]
    fn startup_items() -> Vec<StartupItem> {
        let mut items = Vec::new();
        if let Some(home) = Self::home() {
            items.extend(Self::folder_items(&home.join("Library/LaunchAgents"), ".plist", "Launch agent"));
        }
        items.extend(Self::folder_items(Path::new("/Library/LaunchAgents"), ".plist", "Launch agent"));
        items.extend(Self::folder_items(Path::new("/Library/LaunchDaemons"), ".plist", "Launch daemon"));
        items
    }

    /// Linux: desktop autostart entries and enabled systemd services.
    #[cfg(target_os = "linux")]
    fn startup_items() -> Vec<StartupItem> {
        let mut items = Vec::new();
        if let Some(home) = Self::home() {
            items.extend(Self::folder_items(&home.join(".config/autostart"), ".desktop", "Autostart entry"));
        }
        items.extend(Self::folder_items(Path::new("/etc/xdg/autostart"), ".desktop", "Autostart entry"));
        for (args, kind) in [
            (
                &["list-unit-files", "--state=enabled", "--type=service", "--no-legend", "--no-pager"][..],
                "System service",
            ),
            (
                &["--user", "list-unit-files", "--state=enabled", "--type=service", "--no-legend", "--no-pager"][..],
                "User service",
            ),
        ] {
            let Some(text) = SystemQuery::run("systemctl", args) else { continue };
            items.extend(text.lines().filter_map(|line| {
                let unit = line.split_whitespace().next()?;
                Some(StartupItem {
                    name: unit.trim_end_matches(".service").into(),
                    kind: kind.into(),
                    location: "systemd".into(),
                })
            }));
        }
        items
    }

    /// Windows: the Run keys and the Startup folders.
    #[cfg(target_os = "windows")]
    fn startup_items() -> Vec<StartupItem> {
        let mut items = Vec::new();
        for key in [
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
            r"HKLM\Software\Microsoft\Windows\CurrentVersion\Run",
        ] {
            let Some(text) = SystemQuery::run("reg", &["query", key]) else { continue };
            items.extend(text.lines().filter(|l| l.starts_with("    ")).filter_map(|line| {
                let name = line.trim().split("    ").next()?.trim();
                (!name.is_empty()).then(|| StartupItem {
                    name: name.into(),
                    kind: "Run key".into(),
                    location: key.into(),
                })
            }));
        }
        if let Some(appdata) = std::env::var_os("APPDATA") {
            let folder = PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs\Startup");
            items.extend(Self::folder_items(&folder, ".lnk", "Startup folder"));
        }
        if let Some(programdata) = std::env::var_os("ProgramData") {
            let folder = PathBuf::from(programdata).join(r"Microsoft\Windows\Start Menu\Programs\StartUp");
            items.extend(Self::folder_items(&folder, ".lnk", "Startup folder"));
        }
        items
    }

    /// Other systems: none known.
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn startup_items() -> Vec<StartupItem> {
        Vec::new()
    }
}

impl Probe for SoftwareProbe {
    type Report = SoftwareReport;

    /// Startup items rarely change; once a minute notices a new one soon enough.
    fn interval(&self) -> Duration {
        Duration::from_secs(60)
    }

    /// Reads the system's facts and startup items.
    fn read(&mut self) -> SoftwareReport {
        SoftwareReport {
            system: System::long_os_version().unwrap_or_else(|_| std::env::consts::OS.into()),
            version: System::os_version().unwrap_or_default(),
            kernel: System::kernel_version().unwrap_or_default(),
            host_name: System::host_name().unwrap_or_default(),
            uptime: System::uptime().unwrap_or(0),
            boot_time: System::boot_time().unwrap_or(0),
            startup_items: Self::startup_items(),
        }
    }
}
