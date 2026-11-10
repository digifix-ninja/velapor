// What the window knows: the latest snapshot, the last hour, the screens, and what was
// measured on request during this session. Nothing is saved; closing Velapor forgets it.

export const DOMAINS = [
  { key: "hardware", label: "Hardware" },
  { key: "software", label: "Software" },
  { key: "network", label: "Network" },
  { key: "performance", label: "Performance" },
  { key: "integrity", label: "Integrity" },
];

/** How many samples the trends keep: one hour of two-second beats. */
const HISTORY = 1800;

export class Store {
  /** Starts empty. */
  constructor() {
    this.snapshot = null;
    this.history = [];
    this.displays = [];
    this.preflight = null;
    this.speedTests = [];
    this.listeners = [];
  }

  /** Calls `listener(reason)` after every change. */
  subscribe(listener) {
    this.listeners.push(listener);
  }

  /** Tells every listener that something changed. */
  changed(reason) {
    for (const listener of this.listeners) listener(reason);
  }

  /** Takes a new snapshot and adds its numbers to the history. */
  setSnapshot(snapshot) {
    if (!snapshot) return;
    this.snapshot = snapshot;
    const last = this.history[this.history.length - 1];
    if (!last || last.at < snapshot.at) {
      this.history.push(Store.sampleOf(snapshot));
      if (this.history.length > HISTORY) this.history.shift();
    }
    this.changed("snapshot");
  }

  /** Replaces the history with the one the Rust side kept. */
  setHistory(samples) {
    if (Array.isArray(samples) && samples.length) this.history = samples.slice(-HISTORY);
  }

  /** Remembers the connected screens. */
  setDisplays(displays) {
    this.displays = displays || [];
  }

  /** Remembers the last Preflight result. */
  setPreflight(report) {
    this.preflight = report;
    this.changed("preflight");
  }

  /** Adds a finished speed test to this session's list. */
  addSpeedTest(result) {
    this.speedTests.unshift({ ...result, at: Date.now() });
    this.changed("speed");
  }

  /** The findings of one tab, or all of them. */
  findings(domain = "all") {
    const all = this.snapshot?.findings ?? [];
    return domain === "all" ? all : all.filter(f => f.domain === domain);
  }

  /** The checks of one tab. */
  checks(domain) {
    return (this.snapshot?.checks ?? []).filter(c => c.domain === domain);
  }

  /** The summary of one tab. */
  summary(domain) {
    return this.snapshot?.domains.find(d => d.domain === domain);
  }

  /** Whether a finding with this id prefix is open, e.g. "network.new-listener:TCP:8384". */
  hasFinding(id) {
    return this.findings().some(f => f.id === id);
  }

  /** The numbers a trend needs, taken from a snapshot (same shape as the Rust Sample). */
  static sampleOf(snapshot) {
    const r = snapshot.readings, m = r.processor.memory;
    const hottest = r.sensors.reduce((max, s) => (max === null || s.celsius > max ? s.celsius : max), null);
    return {
      at: snapshot.at,
      cpu: r.processor.cpu.usage,
      memory: m.total ? (m.used / m.total) * 100 : 0,
      swap: m.swapTotal ? (m.swapUsed / m.swapTotal) * 100 : 0,
      received: r.network.receivedPerSec,
      sent: r.network.sentPerSec,
      diskRead: r.performance.diskReadPerSec,
      diskWritten: r.performance.diskWrittenPerSec,
      temperature: hottest,
    };
  }
}
