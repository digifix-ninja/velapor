# Velapor Architecture

This document describes how Velapor is put together. It is a living draft and will change as the project grows.

## 1. The big picture

Velapor is split into two programs that talk to each other.

```
┌──────────────────────────────┐        local socket / named pipe        ┌──────────────────────────────┐
│  Velapor (the app)           │  ◄──────────────────────────────────►   │  Velapor Sentry (service)    │
│  Tauri + web UI              │      versioned, read-mostly API         │  Rust, runs with privileges  │
│  runs as the normal user     │                                         │  collects, stores, analyses  │
│  cockpit, preflight, alerts  │                                         │  probes · rules · history    │
└──────────────────────────────┘                                         └──────────────────────────────┘
```

**Velapor Sentry** is a small background service. Much of what Velapor needs to see (SMART data, temperature sensors, the owner process of every open port, system logs, ICMP for latency) requires elevated rights on every operating system. Sentry is the only part that holds those rights. It collects measurements on a schedule, stores history, runs the diagnostic rules and keeps working when no window is open.

**Velapor** is the app you actually look at. It runs as your normal user, shows the cockpit, runs Preflight on request and displays alerts. It never touches the system directly; it only asks Sentry.

Why the split matters:

- The privileged code stays small, focused and auditable.
- A bug in the UI can never be used to gain elevated rights.
- Monitoring and alerts continue in the background, even with the app closed.

## 2. Repository layout

A single Cargo workspace:

```
velapor/
├── crates/
│   ├── core/       Shared data types: metrics, findings, severities, snapshots
│   ├── probes/     Data collectors, one module per domain, with per-OS backends
│   ├── rules/      The diagnostic engine: turns measurements into findings
│   ├── store/      History in SQLite, with downsampling
│   ├── ipc/        Protocol definitions and client/server transport
│   └── sentry/     The background service binary
├── app/            The Tauri application (Rust backend + web frontend)
├── packaging/      Installers and service definitions per OS
└── docs/
```

## 3. Probes

A **probe** is a small collector that answers one question, for example "what are the current disk temperatures?" or "which ports are listening, and who owns them?".

Every probe implements the same trait:

```rust
/// A collector that measures one aspect of the system.
trait Probe {
    /// Stable identifier, e.g. "hardware.disk.smart".
    fn id(&self) -> &'static str;
    /// How often this probe should run by default.
    fn interval(&self) -> Duration;
    /// Takes one measurement.
    fn sample(&mut self) -> Result<Sample, ProbeError>;
}
```

Each probe has a backend per operating system behind `#[cfg(target_os = ...)]`. When a measurement is not available on a platform, the probe says so honestly ("not supported here") instead of pretending.

### Planned data sources per platform

| Domain | macOS | Windows | Linux |
| --- | --- | --- | --- |
| CPU, memory, processes | `sysinfo` crate, `libproc` | `sysinfo`, Win32 APIs | `sysinfo`, `/proc` |
| Temperatures, fans | SMC / IOKit (IOHID sensors on Apple Silicon) | WMI, vendor interfaces | `hwmon` in `/sys/class/hwmon` |
| Disk SMART | `smartctl --json` (smartmontools, optional) | `smartctl --json` | `smartctl --json` |
| Battery | IOKit `AppleSmartBattery` | WMI, battery report | `/sys/class/power_supply` |
| GPU | IOKit / Metal statistics | NVML (NVIDIA), DXGI | NVML, `/sys/class/drm` |
| Ports and connections | `libproc` socket info | `GetExtendedTcpTable` / `GetExtendedUdpTable` | `/proc/net`, netlink |
| Services | `launchd` | Service Control Manager | `systemd` (D-Bus) |
| Startup items | LaunchAgents/Daemons, login items | Run keys, Startup folder, scheduled tasks | systemd units, XDG autostart, cron |
| Drivers / extensions | system and kernel extensions | driver store | loaded kernel modules |
| Logs | unified log | Event Log | journald |
| Updates | `softwareupdate` | Windows Update API | apt / dnf / pacman / flatpak |
| Security posture | SIP, Gatekeeper, FileVault, firewall | Defender status, BitLocker, firewall | firewall, disk encryption, Secure Boot |

These choices are a starting point. Each one will be verified on real hardware before it is relied on.

## 4. Findings and the rules engine

Raw numbers are not useful on their own. The **rules engine** turns them into **findings** that a human can act on.

```rust
/// Something Velapor wants to tell the user.
struct Finding {
    id: String,              // e.g. "disk.smart.reallocated_sectors"
    severity: Severity,      // Clear, Notice, Warning, Critical
    title: String,           // "Your main drive is reallocating sectors"
    explanation: String,     // What this means, in plain language
    likely_cause: String,    // Why it is probably happening
    recommendation: String,  // What the user can do
    evidence: Vec<Evidence>, // The measurements behind the finding
}
```

Rules come in two kinds:

- **Declarative rules**, written as data (TOML), for simple thresholds and conditions. Easy to read, review and contribute.
- **Code rules**, written in Rust, for anything that needs history, trends or several signals combined (for example "this process's memory has grown steadily for six hours").

Every finding must carry its evidence. If Velapor cannot show why it believes something, it should not say it.

## 5. Preflight

Preflight runs every applicable probe immediately, evaluates every rule and produces one report:

- an overall state: **Clear**, **Keep an eye on it**, or **Needs attention**
- the findings grouped by domain, most important first
- what was checked, and what could not be checked on this machine

Reports are stored, so the user can compare today's Preflight with last week's.

## 6. Baselines and integrity

For system integrity, Sentry keeps **baselines**: snapshots of the parts of the system that should rarely change, such as the hosts file, DNS settings, startup items, services, firewall state, sudoers and security settings. Each snapshot is hashed and compared with the previous one. A change is not automatically bad (the user may have installed something), so Velapor reports what changed and when, and lets the user mark a change as expected.

## 7. Storage and history

History lives in a SQLite database in the system data directory, owned by Sentry.

| Resolution | Kept for |
| --- | --- |
| Raw samples | 24 hours |
| 1-minute averages | 7 days |
| 1-hour averages | 1 year |

This keeps long-term trends available without letting the database grow without limit.

## 8. The IPC channel

The app and Sentry communicate over a local, versioned protocol:

- **Unix domain socket** on macOS and Linux, **named pipe** on Windows. Never a network port.
- Access restricted by file permissions and by checking the peer's identity (`getpeereid` / `SO_PEERCRED` / named pipe client token).
- The API is **read-mostly**: query metrics, query findings, run Preflight, subscribe to live updates. Anything that changes the system (a future "apply fix" feature) will require explicit user confirmation and a separate, clearly marked command.

## 9. Installation and privileges per OS

| | How Sentry runs | How it is installed |
| --- | --- | --- |
| macOS | launchd daemon registered via `SMAppService` | signed and notarised app bundle |
| Windows | Windows service | MSI installer |
| Linux | systemd service, socket access via a `velapor` group | packages (deb, rpm), AppImage for the app |

## 10. Principles in code

- **Local only.** Sentry makes no outbound connections except the ones a check explicitly needs (latency and DNS tests), and these are listed in the settings.
- **Read-only by default.** No probe and no rule changes the system.
- **Small privileged surface.** Sentry has no UI, no web view and no plugin system.
- **Readable code.** Object-based, logically organised, with a short comment on every function.

## 11. Roadmap

| Phase | Goal |
| --- | --- |
| 0 | Workspace skeleton, IPC between app and Sentry, one probe end to end |
| 1 | Hardware and performance probes on macOS, live cockpit |
| 2 | Rules engine, findings and the first Preflight |
| 3 | Network probes: ports, connections, bandwidth, latency, DNS |
| 4 | OS health: services, startup items, logs, updates |
| 5 | Baselines and integrity checks |
| 6 | Windows and Linux backends to parity |
| 7 | History, trends and the menu bar / tray watchman |

## 12. Open questions

- Should the frontend be plain web (Svelte, Solid) or something heavier? Lighter is better for an app that may stay open all day.
- Which SMART approach on each platform: bundle smartmontools, require it, or read it natively?
- Notifications: native OS notifications only, or also an optional local webhook for homelab users?
- How far should "apply fix" ever go, if at all?
