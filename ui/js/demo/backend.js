// A simulated machine for working on the window in a plain browser, and for the website.
// It answers the same commands and sends the same events as the Rust side, with made-up
// readings of a laptop on a desk. Nothing here reads your real machine.

const MINUTE = 60_000;
const GB = 1e9;

/** Rule ids and names, in the order the Rust rule book runs them. */
const RULES = [
  ["hardware", "hardware.disk-space", "Free space on drives"],
  ["hardware", "hardware.temperature", "Temperature"],
  ["hardware", "hardware.battery-health", "Battery health"],
  ["hardware", "hardware.new-device", "Newly attached devices"],
  ["software", "software.uptime", "Time since restart"],
  ["network", "network.new-listener", "New open ports"],
  ["network", "network.risky-port", "Exposed services"],
  ["network", "network.offline", "Network connection"],
  ["performance", "performance.sustained-load", "Lasting processor load"],
  ["performance", "performance.memory-pressure", "Memory pressure"],
  ["performance", "performance.swap", "Swap use"],
  ["performance", "performance.hungry-process", "Processes using a lot of memory"],
  ["integrity", "integrity.posture", "Security settings"],
  ["integrity", "integrity.changed", "Hosts file and DNS servers"],
  ["integrity", "integrity.new-startup-item", "New startup items"],
];

const DOMAIN_LABELS = { hardware: "Hardware", software: "Software", network: "Network", performance: "Performance", integrity: "Integrity" };
const ORDER = { clear: 0, notice: 1, warning: 2, critical: 3 };

/** A value that wanders around a resting point, like a real reading. */
class Wander {
  /** Starts at `rest`, wandering by about `spread`, never below `min` or above `max`. */
  constructor(rest, spread, min = 0, max = Infinity) {
    Object.assign(this, { rest, spread, min, max, value: rest });
  }

  /** Takes one step and returns the new value. */
  step() {
    this.value += (Math.random() - 0.5) * this.spread + (this.rest - this.value) * 0.12;
    if (Math.random() < 0.025) this.value += this.spread * (1 + Math.random());
    this.value = Math.min(this.max, Math.max(this.min, this.value));
    return this.value;
  }
}

/** The pretend machine: a 16-inch laptop with a dock, a drive that is getting full, and a
 *  sync tool that was installed a few minutes after Velapor opened. */
class DemoMachine {
  /** Sets up the resting values. */
  constructor() {
    this.openedAt = Date.now() - 34 * MINUTE;
    this.cpu = new Wander(24, 14, 2, 100);
    this.memory = new Wander(61, 1.5, 40, 90);
    this.heat = new Wander(56, 3, 38, 95);
    this.down = new Wander(2.4e6, 1.6e6);
    this.up = new Wander(0.35e6, 0.3e6);
    this.read = new Wander(8e6, 9e6);
    this.write = new Wander(3e6, 4e6);
    this.newItemAt = this.openedAt + 21 * MINUTE;
  }

  /** One fresh set of readings. */
  readings() {
    const memTotal = 32 * GB, memUsed = (this.memory.step() / 100) * memTotal;
    const heat = this.heat.step();
    const down = this.down.step(), up = this.up.step();
    return {
      machine: {
        model: "MacBook Pro 16-inch (2023)", maker: "Apple", modelId: "Mac14,6", processor: "Apple M2 Max",
        cores: 12, physicalCores: 12, memory: memTotal, graphics: ["Apple M2 Max (38 cores)"],
        system: "macOS 15.4", kernel: "Darwin 24.4.0", hostName: "studio-mbp", architecture: "arm64",
      },
      processor: {
        cpu: { usage: this.cpu.step(), cores: 12, physicalCores: 12, frequencyMhz: 3700, brand: "Apple M2 Max", loadAverage: [2.84, 2.61, 2.4] },
        memory: { total: memTotal, used: memUsed, available: memTotal - memUsed, swapTotal: 2 * GB, swapUsed: 0.41 * GB },
      },
      disks: [
        { name: "Macintosh HD", mountPoint: "/", fileSystem: "apfs", kind: "SSD", total: 2000 * GB, available: 171 * GB, removable: false, readOnly: false, readPerSec: this.read.step(), writtenPerSec: this.write.step() },
        { name: "T7", mountPoint: "/Volumes/T7", fileSystem: "exfat", kind: "SSD", total: 1000 * GB, available: 612 * GB, removable: true, readOnly: false, readPerSec: 0, writtenPerSec: 0 },
      ],
      sensors: [
        { label: "CPU die", celsius: heat, highest: 71, critical: 105 },
        { label: "GPU", celsius: heat - 6, highest: 64, critical: 105 },
        { label: "SSD", celsius: 41 + Math.random(), highest: 46, critical: 85 },
        { label: "Battery", celsius: 31 + Math.random() * 0.5, highest: 33, critical: 60 },
      ],
      battery: { charge: 78, health: 87, cycles: 412, state: "Charging", temperature: 31, model: "bq40z651", technology: "Lithium-ion" },
      devices: this.devices(),
      network: {
        interfaces: [
          { name: "en0", mac: "a4:83:e7:1c:02:9f", addresses: ["192.168.178.24", "fd00::1c8a:7e2"], receivedPerSec: down, sentPerSec: up, totalReceived: 8.4 * GB, totalSent: 1.1 * GB },
          { name: "utun3", mac: "", addresses: ["10.8.0.6"], receivedPerSec: 1200, sentPerSec: 900, totalReceived: 0.2 * GB, totalSent: 0.1 * GB },
        ],
        receivedPerSec: down, sentPerSec: up,
        listening: this.ports(),
        established: 38,
        busiest: [
          { process: "Safari", count: 12, remotes: ["17.253.144.10", "151.101.1.69", "104.16.132.229"] },
          { process: "syncthing", count: 4, remotes: ["192.168.178.40", "139.162.130.104"] },
          { process: "Mail", count: 3, remotes: ["17.42.251.41"] },
        ],
      },
      performance: {
        processCount: 641,
        byCpu: [
          { pid: 412, name: "WindowServer", cpu: 6 + Math.random() * 4, memory: 0.61 * GB, readPerSec: 0, writtenPerSec: 2e4, runTime: 302_400 },
          { pid: 2211, name: "Xcode", cpu: 4 + Math.random() * 6, memory: 2.1 * GB, readPerSec: 4e6, writtenPerSec: 8e5, runTime: 7_400 },
          { pid: 1880, name: "Safari", cpu: 3 + Math.random() * 3, memory: 1.4 * GB, readPerSec: 6e5, writtenPerSec: 1e5, runTime: 21_600 },
          { pid: 990, name: "mds_stores", cpu: 2 + Math.random() * 3, memory: 0.19 * GB, readPerSec: 1.2e7, writtenPerSec: 2e5, runTime: 840 },
          { pid: 3302, name: "syncthing", cpu: 1 + Math.random() * 2, memory: 0.1 * GB, readPerSec: 1.1e6, writtenPerSec: 9e5, runTime: 780 },
        ],
        byMemory: [
          { pid: 2211, name: "Xcode", cpu: 6.1, memory: 2.1 * GB, readPerSec: 0, writtenPerSec: 0, runTime: 7_400 },
          { pid: 1880, name: "Safari", cpu: 3.2, memory: 1.4 * GB, readPerSec: 0, writtenPerSec: 0, runTime: 21_600 },
          { pid: 2404, name: "Figma", cpu: 1.1, memory: 1.1 * GB, readPerSec: 0, writtenPerSec: 0, runTime: 9_000 },
          { pid: 412, name: "WindowServer", cpu: 7.4, memory: 0.61 * GB, readPerSec: 0, writtenPerSec: 0, runTime: 302_400 },
        ],
        diskReadPerSec: this.read.value, diskWrittenPerSec: this.write.value,
      },
      software: {
        system: "macOS", version: "15.4", kernel: "Darwin 24.4.0", hostName: "studio-mbp",
        uptime: 3 * 86400 + 19 * 3600, bootTime: Math.floor(Date.now() / 1000) - (3 * 86400 + 19 * 3600),
        startupItems: this.startupItems(),
      },
      posture: [
        { key: "system-protection", name: "System Integrity Protection", state: "on", detail: "System Integrity Protection status: enabled." },
        { key: "app-verification", name: "Gatekeeper", state: "on", detail: "assessments enabled" },
        { key: "disk-encryption", name: "FileVault", state: "on", detail: "FileVault is On." },
        { key: "firewall", name: "Firewall", state: "on", detail: "Firewall is enabled. (State = 1)" },
      ],
      fingerprints: [
        { key: "hosts-file", name: "Hosts file", hash: "4f1c09a2be771d03", summary: "3 entries" },
        { key: "dns-servers", name: "DNS servers", hash: "9a0d2c11f7e4b812", summary: "192.168.178.1" },
      ],
    };
  }

  /** Things plugged in. */
  devices() {
    return [
      { key: "usb:2109:0820", name: "CalDigit TS4", maker: "CalDigit", kind: "Hub", connection: "USB 10 Gb/s", detail: "Thunderbolt dock · 18 ports" },
      { key: "usb:04e8:4001", name: "Portable SSD T7", maker: "Samsung", kind: "Storage", connection: "USB 10 Gb/s", detail: "Mass storage" },
      { key: "usb:1050:0407", name: "YubiKey 5C NFC", maker: "Yubico", kind: "Security key", connection: "USB 12 Mb/s", detail: "FIDO2 · OTP · smart card" },
      { key: "bt:magic-keyboard", name: "Magic Keyboard with Touch ID", maker: "Apple", kind: "Keyboard", connection: "Bluetooth", detail: "battery 64 %" },
      { key: "bt:mx-master", name: "MX Master 3S", maker: "Logitech", kind: "Mouse", connection: "Bluetooth", detail: "battery 81 %" },
      { key: "power:adapter", name: "140W USB-C Power Adapter", maker: "Apple", kind: "Power", connection: "Charging port", detail: "140 W" },
    ];
  }

  /** Programs waiting for connections; syncthing arrived after Velapor opened. */
  ports() {
    const ports = [
      { protocol: "TCP", port: 5000, address: "0.0.0.0", reachableFromNetwork: true, process: "ControlCenter", pid: 512 },
      { protocol: "TCP", port: 7000, address: "0.0.0.0", reachableFromNetwork: true, process: "ControlCenter", pid: 512 },
      { protocol: "TCP", port: 5432, address: "127.0.0.1", reachableFromNetwork: false, process: "postgres", pid: 880 },
    ];
    if (Date.now() >= this.newItemAt) ports.push({ protocol: "TCP", port: 8384, address: "127.0.0.1", reachableFromNetwork: false, process: "syncthing", pid: 3302 });
    return ports;
  }

  /** What starts with the system; one of them is new. */
  startupItems() {
    const items = [
      { name: "com.docker.vmnetd", kind: "Launch daemon", location: "/Library/LaunchDaemons" },
      { name: "com.charonsync.agent", kind: "Launch agent", location: "~/Library/LaunchAgents" },
      { name: "com.google.keystone.agent", kind: "Launch agent", location: "~/Library/LaunchAgents" },
    ];
    if (Date.now() >= this.newItemAt) items.push({ name: "net.syncthing", kind: "Launch agent", location: "~/Library/LaunchAgents" });
    return items;
  }

  /** The findings the rules would report for these readings. */
  findings(r) {
    const found = [];
    const root = r.disks[0];
    const used = ((root.total - root.available) / root.total) * 100;
    found.push({
      id: "hardware.disk-space:/", domain: "hardware", severity: "warning", title: `Macintosh HD is ${used.toFixed(0)} % full`,
      explanation: "When a drive runs out of space, programs cannot save their work and the system slows down or stops updating.",
      likelyCause: "Large downloads, caches, old backups or virtual machines usually take the most room.",
      recommendation: "Look at what takes the most space on this drive and move or remove what you no longer need.",
      evidence: [{ label: "Drive", value: "/" }, { label: "Free", value: "171 GB of 2 TB" }], since: this.openedAt,
    });
    if (Date.now() >= this.newItemAt) {
      found.push({
        id: "integrity.new-startup-item:Launch agent|~/Library/LaunchAgents|net.syncthing", domain: "integrity", severity: "notice",
        title: "net.syncthing now starts with the system",
        explanation: "A program added itself to the list of things that start automatically. Installers do this; unwanted software does too.",
        likelyCause: null, recommendation: "If you just installed something, this is expected. If not, look up the name before you leave it.",
        evidence: [{ label: "Item", value: "net.syncthing" }, { label: "Kind", value: "Launch agent" }, { label: "Where", value: "~/Library/LaunchAgents" }], since: this.newItemAt,
      });
      found.push({
        id: "network.new-listener:TCP:8384:syncthing", domain: "network", severity: "notice", title: "syncthing opened port 8384",
        explanation: "A program started waiting for incoming connections. Developer tools and sharing features do this; unwanted software does too.",
        likelyCause: null, recommendation: "If you recognise the program, this is fine. If not, find out what it is before you leave it running.",
        evidence: [{ label: "Port", value: "TCP 8384" }, { label: "Listening on", value: "127.0.0.1" }, { label: "Reachable from", value: "this machine only" }], since: this.newItemAt + 4000,
      });
    }
    return found.sort((a, b) => ORDER[b.severity] - ORDER[a.severity]);
  }

  /** A full snapshot, shaped like the Rust one. */
  snapshot() {
    const readings = this.readings();
    const findings = this.findings(readings);
    const checks = RULES.map(([domain, id, name]) => {
      const mine = findings.filter(f => f.id.startsWith(id));
      const severity = mine.reduce((worst, f) => (ORDER[f.severity] > ORDER[worst] ? f.severity : worst), "clear");
      return { id, domain, name, severity, applicable: true };
    });
    const domains = Object.keys(DOMAIN_LABELS).map(domain => {
      const mine = findings.filter(f => f.domain === domain);
      const rules = checks.filter(c => c.domain === domain);
      return {
        domain, label: DOMAIN_LABELS[domain], findings: mine.length, checks: rules.length,
        clear: rules.filter(c => c.severity === "clear").length,
        severity: mine.reduce((worst, f) => (ORDER[f.severity] > ORDER[worst] ? f.severity : worst), "clear"),
      };
    });
    const worst = findings[0];
    const headline = worst
      ? { severity: worst.severity, title: worst.title, detail: findings.length > 1 ? `There ${findings.length === 2 ? "is one more thing" : `are ${findings.length - 1} more things`} below.` : "This is the only thing Velapor would look at." }
      : { severity: "clear", title: "All quiet", detail: "Everything Velapor watches looks fine." };
    return { at: Date.now(), watchingSince: this.openedAt, headline, domains, findings, checks, readings };
  }

  /** Half an hour of made-up samples, so the trends have a past. */
  history() {
    const samples = [];
    for (let at = this.openedAt; at < Date.now(); at += 2000) {
      samples.push({
        at, cpu: this.cpu.step(), memory: this.memory.step(), swap: 20, received: this.down.step(), sent: this.up.step(),
        diskRead: this.read.step(), diskWritten: this.write.step(), temperature: this.heat.step(),
      });
    }
    return samples;
  }
}

/** Answers the window's commands like the Rust side would. */
export class DemoBackend {
  /** Creates the backend in the shape of `window.__TAURI__`. */
  static create() {
    const backend = new DemoBackend();
    return {
      core: { invoke: (command, args) => backend.invoke(command, args) },
      event: { listen: async (event, handler) => backend.listen(event, handler) },
    };
  }

  /** Starts the simulated beat. */
  constructor() {
    this.machine = new DemoMachine();
    this.listeners = {};
    this.pastSamples = this.machine.history();
    this.latest = this.machine.snapshot();
    setInterval(() => {
      this.latest = this.machine.snapshot();
      this.emit("snapshot", this.latest);
    }, 2000);
  }

  /** Registers an event handler and returns a function that removes it. */
  listen(event, handler) {
    (this.listeners[event] ??= []).push(handler);
    return () => { this.listeners[event] = this.listeners[event].filter(h => h !== handler); };
  }

  /** Sends an event to every handler. */
  emit(event, payload) {
    for (const handler of this.listeners[event] ?? []) handler({ payload });
  }

  /** Runs a command. */
  async invoke(command) {
    switch (command) {
      case "get_snapshot": return this.latest;
      case "get_history": return this.pastSamples;
      case "get_displays": return [
        { key: "display:built-in", name: "Built-in Liquid Retina XDR", maker: null, kind: "Main display", connection: "Display", detail: "3456 × 2234 · scale 2" },
        { key: "display:lg", name: "LG UltraFine 5K", maker: null, kind: "Display", connection: "Display", detail: "5120 × 2880 · scale 2" },
      ];
      case "run_preflight": return this.preflight();
      case "run_speed_test": return this.speedTest();
      case "about": return {
        name: "Velapor", version: "1.0.0", appWebsite: "https://velapor.app", developer: "DigiFix S.à r.l.-S.", website: "https://digifix.ninja",
        family: "https://digifix.ninja", sourceCode: "https://github.com/digifix-ninja/velapor", licence: "MIT", copyrightYear: "2026",
      };
      case "open_url": return null;
      default: throw new Error(`The demo doesn't know "${command}".`);
    }
  }

  /** A pretend Preflight that takes a moment, like the real one. */
  async preflight() {
    await DemoBackend.wait(2200);
    const s = this.machine.snapshot();
    return {
      at: Date.now(), tookMs: 2200, verdict: s.headline, checks: s.checks, findings: s.findings,
      connection: { names: { ok: true, millis: 18, detail: "Names are resolved" }, internet: { ok: true, millis: 14, detail: "The internet answers" } },
    };
  }

  /** A pretend speed test that sends progress events. */
  async speedTest() {
    const result = { latencyMs: 11 + Math.random() * 8, jitterMs: 1 + Math.random() * 3, downloadMbps: 330 + Math.random() * 140, uploadMbps: 44 + Math.random() * 6, server: "speed.cloudflare.com" };
    for (let i = 1; i <= 8; i++) { await DemoBackend.wait(90); this.emit("speed-progress", { phase: "latency", fraction: i / 8, value: result.latencyMs * (0.9 + Math.random() * 0.2) }); }
    for (let i = 1; i <= 20; i++) { await DemoBackend.wait(90); this.emit("speed-progress", { phase: "download", fraction: i / 20, value: result.downloadMbps * Math.min(1, i / 8) * (0.92 + Math.random() * 0.12) }); }
    for (let i = 1; i <= 16; i++) { await DemoBackend.wait(90); this.emit("speed-progress", { phase: "upload", fraction: i / 16, value: result.uploadMbps * Math.min(1, i / 6) * (0.92 + Math.random() * 0.12) }); }
    return result;
  }

  /** Waits `ms` milliseconds. */
  static wait(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}
