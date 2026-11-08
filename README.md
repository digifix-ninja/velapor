# Velapor

**Vela por ti.** *It watches over you.*

Velapor is an open-source cockpit for your computer. It quietly keeps watch over your hardware, network, operating system and performance, and when something looks wrong, it tells you what is happening, why it is probably happening, and what you can do about it.

It is not an antivirus. It does not hunt for malware signatures or quarantine files. Think of it as the instrument panel of an aircraft combined with a mechanic's diagnostic equipment: calm gauges while everything is fine, a clear warning light when something needs your attention, and a full preflight inspection whenever you want one.

> **Status: early development.** The architecture is being designed and nothing is usable yet. Watch or star the repository to follow along.

---

## What Velapor watches

**Hardware health**
CPU, GPU, memory and storage load, temperatures, fan speeds, SMART data from your drives, battery condition, and sensor readings that drift out of their normal range.

**Network health**
Open ports, active connections and which process owns them, bandwidth, latency, DNS resolution, unusual traffic patterns and a history of connections.

**Operating system health**
Services, drivers and system extensions, startup items, system logs, pending updates, permissions and changes to important configuration.

**Performance**
Bottlenecks, resource-hungry processes, disk I/O, memory pressure, process behaviour and long-term trends, so you can see whether your machine is slowly getting worse.

**System integrity**
Unexpected modifications to critical settings, filesystem errors, failing services and changes to the parts of the system that should normally stay still.

**Diagnostics**
Every warning comes with an explanation in plain language, the most likely cause, and a recommended fix. Velapor explains; it never acts behind your back.

## Preflight

Pilots walk around the aircraft before every flight. Velapor's **Preflight** does the same for your computer: one button runs every check and gives you a single, readable report. Green means clear skies. Amber means keep an eye on it. Red means look at this now.

## Principles

- **Local only.** Everything Velapor sees stays on your machine. No cloud, no account, no telemetry.
- **Read-only by default.** Velapor observes and recommends. It never changes your system unless you explicitly ask it to.
- **Least privilege.** Only a small, auditable background service runs with elevated rights. The app you look at runs as your normal user.
- **Explainable.** No mysterious scores. Every finding shows the evidence it is based on.
- **Calm.** A watchman, not an alarm siren. Velapor speaks up when it matters and stays quiet when it does not.

## Platforms

macOS, Windows and Linux, built with Rust and Tauri.

## The name

*Velar por* is Spanish and Portuguese for "to watch over" or "to look after". *Vela por ti* means "it watches over you". A *vela* is also a candle: a small light that keeps burning through the night. That is exactly what Velapor wants to be for your computer.

## Contributing

Velapor is at the very beginning. Ideas, questions and discussions are welcome in the issues. Code contributions will open up once the foundation is in place. See [ARCHITECTURE.md](ARCHITECTURE.md) for how the project is put together.

## Licence

[MIT](LICENSE)

---

[velapor.app](https://velapor.app)
