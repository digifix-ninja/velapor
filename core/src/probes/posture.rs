//! Security settings: the firewall, disk encryption and the system's own protections.
//! Each is asked from the system's own reporting tool, and only read.

use super::Probe;
#[allow(unused_imports)]
use crate::system_query::SystemQuery;
use serde::Serialize;
use std::time::Duration;

/// Whether a protection is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PostureState {
    /// The protection is on.
    On,
    /// The protection is off.
    Off,
    /// The system didn't say (often because it needs administrator rights to ask).
    Unknown,
}

/// One security setting.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostureCheck {
    /// Stable key, e.g. "firewall".
    pub key: String,
    /// Its name, e.g. "FileVault".
    pub name: String,
    /// On, off or unknown.
    pub state: PostureState,
    /// A short detail, e.g. "Stealth mode on".
    pub detail: String,
}

impl PostureCheck {
    /// Creates a check.
    #[allow(dead_code)]
    fn new(key: &str, name: &str, state: PostureState, detail: impl Into<String>) -> Self {
        Self { key: key.into(), name: name.into(), state, detail: detail.into() }
    }

    /// A check whose state is read from a tool's answer: on when it contains `on_word`.
    #[allow(dead_code)]
    fn from_answer(key: &str, name: &str, answer: Option<String>, on_word: &str) -> Self {
        let state = match answer {
            Some(text) if text.to_lowercase().contains(&on_word.to_lowercase()) => PostureState::On,
            Some(_) => PostureState::Off,
            None => PostureState::Unknown,
        };
        let detail = match state {
            PostureState::On => "On",
            PostureState::Off => "Off",
            PostureState::Unknown => "Couldn't be read",
        };
        Self::new(key, name, state, detail)
    }
}

/// Reads the security settings.
pub struct PostureProbe;

impl PostureProbe {
    /// macOS: System Integrity Protection, Gatekeeper, FileVault and the firewall.
    #[cfg(target_os = "macos")]
    fn checks() -> Vec<PostureCheck> {
        vec![
            PostureCheck::from_answer(
                "system-protection",
                "System Integrity Protection",
                SystemQuery::run_all("csrutil", &["status"]),
                "enabled",
            ),
            PostureCheck::from_answer(
                "app-verification",
                "Gatekeeper",
                SystemQuery::run_all("spctl", &["--status"]),
                "assessments enabled",
            ),
            PostureCheck::from_answer(
                "disk-encryption",
                "FileVault",
                SystemQuery::run_all("fdesetup", &["status"]),
                "FileVault is On",
            ),
            PostureCheck::from_answer(
                "firewall",
                "Firewall",
                SystemQuery::run_all("/usr/libexec/ApplicationFirewall/socketfilterfw", &["--getglobalstate"]),
                "enabled",
            ),
        ]
    }

    /// Linux: a running firewall service, encrypted disks and Secure Boot.
    #[cfg(target_os = "linux")]
    fn checks() -> Vec<PostureCheck> {
        let firewall = ["firewalld", "ufw", "nftables"].iter().find(|service| {
            SystemQuery::run("systemctl", &["is-active", service]).is_some_and(|s| s.trim() == "active")
        });
        let firewall_state = match firewall {
            Some(name) => PostureCheck::new("firewall", "Firewall", PostureState::On, format!("{name} is running")),
            None if SystemQuery::run("systemctl", &["--version"]).is_some() => {
                PostureCheck::new("firewall", "Firewall", PostureState::Off, "No firewall service is running")
            }
            None => PostureCheck::new("firewall", "Firewall", PostureState::Unknown, "Couldn't be read"),
        };
        let encryption = match SystemQuery::run("lsblk", &["-n", "-o", "TYPE"]) {
            Some(text) if text.lines().any(|l| l.trim() == "crypt") => PostureCheck::new(
                "disk-encryption",
                "Disk encryption",
                PostureState::On,
                "An encrypted volume is in use",
            ),
            Some(_) => {
                PostureCheck::new("disk-encryption", "Disk encryption", PostureState::Off, "No encrypted volume found")
            }
            None => PostureCheck::new("disk-encryption", "Disk encryption", PostureState::Unknown, "Couldn't be read"),
        };
        let secure_boot = PostureCheck::from_answer(
            "secure-boot",
            "Secure Boot",
            SystemQuery::run_all("mokutil", &["--sb-state"]),
            "SecureBoot enabled",
        );
        vec![firewall_state, encryption, secure_boot]
    }

    /// Windows: the firewall, Microsoft Defender and BitLocker on the system drive.
    #[cfg(target_os = "windows")]
    fn checks() -> Vec<PostureCheck> {
        let firewall = match SystemQuery::run("netsh", &["advfirewall", "show", "allprofiles", "state"]) {
            Some(text) => {
                let on = text.lines().filter(|l| l.trim_end().ends_with("ON")).count();
                let off = text.lines().filter(|l| l.trim_end().ends_with("OFF")).count();
                if off == 0 && on > 0 {
                    PostureCheck::new("firewall", "Firewall", PostureState::On, "On for every network profile")
                } else if on == 0 {
                    PostureCheck::new("firewall", "Firewall", PostureState::Off, "Off for every network profile")
                } else {
                    PostureCheck::new(
                        "firewall",
                        "Firewall",
                        PostureState::Off,
                        format!("Off for {off} network profile(s)"),
                    )
                }
            }
            None => PostureCheck::new("firewall", "Firewall", PostureState::Unknown, "Couldn't be read"),
        };
        let defender = PostureCheck::from_answer(
            "antivirus",
            "Microsoft Defender",
            SystemQuery::powershell("(Get-MpComputerStatus).RealTimeProtectionEnabled"),
            "True",
        );
        let bitlocker = match SystemQuery::powershell(
            "(New-Object -ComObject Shell.Application).NameSpace('C:').Self.ExtendedProperty('System.Volume.BitLockerProtection')",
        ) {
            Some(v) if v == "1" || v == "3" || v == "5" => PostureCheck::new("disk-encryption", "BitLocker", PostureState::On, "System drive encrypted"),
            Some(v) if v == "2" || v == "0" => PostureCheck::new("disk-encryption", "BitLocker", PostureState::Off, "System drive not encrypted"),
            _ => PostureCheck::new("disk-encryption", "BitLocker", PostureState::Unknown, "Couldn't be read"),
        };
        vec![firewall, defender, bitlocker]
    }

    /// Other systems: nothing known.
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn checks() -> Vec<PostureCheck> {
        Vec::new()
    }
}

impl Probe for PostureProbe {
    type Report = Vec<PostureCheck>;

    /// Settings change rarely; every five minutes.
    fn interval(&self) -> Duration {
        Duration::from_secs(300)
    }

    /// Reads each security setting.
    fn read(&mut self) -> Vec<PostureCheck> {
        Self::checks()
    }
}
