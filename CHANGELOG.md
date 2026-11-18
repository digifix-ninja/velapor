# Changelog

All notable changes to Velapor are listed here.

## Unreleased

- Velapor is now made by DigiFix S.à r.l.-S.: new home at github.com/digifix-ninja/velapor, and the About panel, licence and installers carry the DigiFix name.
- New app identifier `ninja.digifix.velapor` (was `dev.stean.velapor`), matching the other DigiFix apps. Velapor stores nothing, so nothing has to move; macOS asks once more whether Velapor may send notifications.

## 0.1.0 — first light

- Six tabs: **Overview**, **Hardware**, **Software**, **Network**, **Performance** and **Integrity**.
- A fresh reading every two seconds, with half an hour of history for the trends.
- Hardware in full: processor, memory, drives, sensors, battery, and attached USB devices, adapters, displays and power.
- Rules for full drives, heat, battery wear, sustained load, memory pressure, swap, hungry processes, risky open ports, new listeners, new startup items, new devices, a changed hosts file or DNS servers, and security settings that are off.
- **Preflight**: every check at once, with name resolution and an internet check, in one report.
- A network **speed test** against Cloudflare: latency, jitter, download and upload.
- Findings with evidence and advice, in four severities: clear, notice, warning and critical.
- Menu bar / system tray icon with the overall state, and notifications for warnings and critical findings.
- Read-only throughout: Velapor never writes, changes or deletes anything.
- macOS, Windows and Linux.
