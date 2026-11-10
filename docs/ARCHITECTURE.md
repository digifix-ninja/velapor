# How Velapor is built

Velapor is a [Tauri 2](https://tauri.app) desktop app in three parts:

- **`core/`** reads the machine and judges what it reads. It has no window and no Tauri in it.
- **`src-tauri/`** is the thin desktop app around the core: the window, the tray icon,
  notifications and the commands the window can call.
- **`ui/`** is the window itself, in plain HTML, CSS and JavaScript modules.

## From a reading to a finding

Every two seconds the app asks the `Watchman` for a `tick`. One tick goes through four steps:

1. **Probes read.** Each probe in `core/src/probes/` reads one part of the machine (processor,
   storage, sensors, battery, devices, network, processes, software, security settings,
   fingerprints). Probes run on their own schedule: load every tick, drives every few seconds,
   startup items and security settings much less often. Together they fill one `Readings`.
2. **History remembers.** A small sample (load, memory, traffic, temperature) goes into the
   `History`, which keeps the last half hour in memory for the trends and for rules that need an
   average over time.
3. **Rules judge.** The `RuleBook` (`core/src/rules/`) runs every `Rule` against a `Context`: the
   readings, the history and the `Baseline`. A rule answers *clear*, *not applicable on this
   machine*, or one or more `Finding`s. A finding has a severity, a title, an explanation, the
   likely cause, a recommendation and the evidence it is based on.
4. **The ledger dates.** The `Ledger` remembers when each finding first appeared, so the window
   can say "since 14:02" and the app can tell a new finding from one it already announced.

The result is a `Snapshot` (headline, one summary per tab, findings, checks and readings) plus
the list of new alerts. The app sends the snapshot to the window, updates the tray, and turns new
warnings and critical findings into notifications.

**The baseline.** On the first tick the watchman captures what should stay still: startup items,
listening ports, attached devices, and fingerprints of the hosts file and DNS servers. Later
ticks compare against it, so a new startup item or a new open port becomes a finding. The
baseline lives in memory only, and a fresh one is captured each time Velapor opens.

**Preflight** runs every probe and rule at once, outside the usual schedule, and adds two
connection checks (`core/src/net/reachability.rs`): whether a name resolves, and whether the
internet answers.

**The speed test** (`core/src/net/speed_test.rs`) measures latency and jitter, then download and
upload against Cloudflare's speed test service. It only runs when the person presses the button.

## Read-only, and how that is kept

Velapor observes and warns, and nothing else. That rule is built into the structure:

- The core reads through libraries that only read (`sysinfo`, `netstat2`, `nusb`,
  `starship-battery`) and by reading a few files and folders (the hosts file, `/sys`, the autostart folders).
- Some facts are only known to the system's own reporting tools. `SystemQuery`
  (`core/src/system_query.rs`) is the only place that may start another program, and only one on
  its fixed list, with arguments fixed in the code. It may stop a tool it started itself if the
  tool takes too long.
- `core/tests/read_only.rs` scans the core's source for write calls (file writes, removals,
  renames, permission changes, process control) and fails if it finds one.
- The app's commands only read. The single exception that leaves the app is opening a link in
  the browser, and only for the app's own fixed links.

## The window

`ui/js/main.js` creates the `App`, which connects to the backend through `Api` and keeps the
latest snapshot in the `Store`. Each tab is a `View` in `ui/js/views/` that redraws itself from
the store when a new snapshot arrives. Widgets (`Gauge`, `Trend`, `SpeedDial`) draw the
instruments.

Outside the app (in a plain browser), `Api` talks to `ui/js/demo/backend.js` instead: a simulated
machine with the same commands and events as the Rust side, for working on the interface without
building the app.

## Per system

| | macOS | Windows | Linux |
|---|---|---|---|
| Security settings | System Integrity Protection, Gatekeeper, FileVault, firewall | Microsoft Defender, firewall, BitLocker | firewall service, disk encryption, Secure Boot |
| Startup items | Launch agents and daemons | Run keys, Startup folders | systemd units, autostart entries |
| Tray icon | menu bar (black, tinted by macOS) | system tray (white) | system tray (white) |
| Closing the window | keeps watching in the menu bar | keeps watching in the tray | quits |
