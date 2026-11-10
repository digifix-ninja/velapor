//! Fingerprints of files and settings that should normally stay the same: the hosts file and
//! the DNS servers. Velapor compares them with how they looked when it started.

use super::Probe;
#[allow(unused_imports)]
use crate::system_query::SystemQuery;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::time::Duration;

/// A short fingerprint of one thing, and a readable summary of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fingerprint {
    /// Stable key, e.g. "hosts-file".
    pub key: String,
    /// Its name, e.g. "Hosts file".
    pub name: String,
    /// A hash of its contents; changes when the contents change.
    pub hash: String,
    /// What it currently says, in short (e.g. the DNS servers).
    pub summary: String,
}

impl Fingerprint {
    /// Fingerprints `contents` under a key and name.
    fn of(key: &str, name: &str, contents: &str, summary: impl Into<String>) -> Self {
        let hash = Sha256::digest(contents.as_bytes());
        let hash = hash.iter().take(8).map(|b| format!("{b:02x}")).collect();
        Self { key: key.into(), name: name.into(), hash, summary: summary.into() }
    }
}

/// Reads the fingerprints.
pub struct FingerprintProbe;

impl FingerprintProbe {
    /// The hosts file, which can quietly send names to other addresses.
    fn hosts_file() -> Option<Fingerprint> {
        let path = if cfg!(windows) { r"C:\Windows\System32\drivers\etc\hosts" } else { "/etc/hosts" };
        let text = std::fs::read_to_string(path).ok()?;
        let entries = text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).count();
        Some(Fingerprint::of("hosts-file", "Hosts file", &text, format!("{entries} entries")))
    }

    /// The DNS servers in use, which decide where names are looked up.
    fn dns_servers() -> Option<Fingerprint> {
        let mut servers = Self::nameservers();
        servers.sort();
        servers.dedup();
        if servers.is_empty() {
            return None;
        }
        let joined = servers.join(", ");
        Some(Fingerprint::of("dns-servers", "DNS servers", &joined, joined.clone()))
    }

    /// Name servers from macOS's resolver report.
    #[cfg(target_os = "macos")]
    fn nameservers() -> Vec<String> {
        SystemQuery::run("scutil", &["--dns"])
            .map(|text| {
                text.lines()
                    .filter_map(|l| {
                        l.trim()
                            .strip_prefix("nameserver[")
                            .and_then(|r| r.split(':').nth(1))
                            .map(|s| s.trim().to_string())
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Name servers from Linux's resolver file.
    #[cfg(target_os = "linux")]
    fn nameservers() -> Vec<String> {
        std::fs::read_to_string("/etc/resolv.conf")
            .map(|text| {
                text.lines().filter_map(|l| l.trim().strip_prefix("nameserver").map(|s| s.trim().to_string())).collect()
            })
            .unwrap_or_default()
    }

    /// Name servers from Windows' network settings.
    #[cfg(target_os = "windows")]
    fn nameservers() -> Vec<String> {
        SystemQuery::powershell("(Get-DnsClientServerAddress -AddressFamily IPv4).ServerAddresses -join ','")
            .map(|text| text.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
            .unwrap_or_default()
    }

    /// Other systems: unknown.
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn nameservers() -> Vec<String> {
        Vec::new()
    }
}

impl Probe for FingerprintProbe {
    type Report = Vec<Fingerprint>;

    /// Once a minute is enough to notice a change.
    fn interval(&self) -> Duration {
        Duration::from_secs(60)
    }

    /// Reads every fingerprint that can be read here.
    fn read(&mut self) -> Vec<Fingerprint> {
        [Self::hosts_file(), Self::dns_servers()].into_iter().flatten().collect()
    }
}
