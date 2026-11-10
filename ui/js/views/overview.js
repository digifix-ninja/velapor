// The Overview tab: the four instruments people glance at, the last hour, the latest
// Preflight and every finding.

import { Dom } from "../dom.js";
import { Format } from "../format.js";
import { Icons } from "../icons.js";
import { Gauge } from "../widgets/gauge.js";
import { Trend } from "../widgets/trend.js";
import { View } from "./view.js";
import { FindingList } from "./finding-list.js";
import { Instruments } from "./instruments.js";

const { html, raw } = Dom;

export class OverviewView extends View {
  /** Builds the gauges and trends. */
  constructor(store) {
    super(store, "overview");
    this.gauges = {
      cpu: new Gauge({ name: "Processor", unit: "%", warn: 85, crit: 95 }),
      memory: new Gauge({ name: "Memory", unit: "%", warn: 90, crit: 97 }),
      heat: new Gauge({ name: "Temperature", unit: "°C", max: 110, warn: 85, crit: 95 }),
      disk: new Gauge({ name: "Fullest drive", unit: "%", warn: 90, crit: 97 }),
    };
    this.trends = [
      Instruments.cpuTrend(),
      Instruments.memoryTrend(),
      new Trend({ name: "Network", unit: "Mb/s", pick: s => Format.megabits(s.received + s.sent), show: v => v.toFixed(1) }),
      new Trend({ name: "Drive activity", unit: "MB/s", pick: s => (s.diskRead + s.diskWritten) / 1e6, show: v => v.toFixed(1), color: "--flame" }),
    ];
    Object.values(this.gauges).forEach(g => this.$("[data-gauges]").appendChild(g.element));
    this.trends.forEach(t => this.$("[data-trends]").appendChild(t.element));
    this.findingList = new FindingList(this.$("[data-findings]"));
  }

  /** Draws the tab. */
  render() {
    const snapshot = this.store.snapshot;
    if (!snapshot) return;
    Instruments.setCpu(this.gauges.cpu, snapshot.readings);
    Instruments.setMemory(this.gauges.memory, snapshot.readings);
    Instruments.setHeat(this.gauges.heat, snapshot.readings);
    Instruments.setFullestDisk(this.gauges.disk, snapshot.readings);
    this.findingList.render(this.store.findings(), "Nothing to report. Everything Velapor watches looks fine.");
    this.renderPreflight();
    this.repaint();
  }

  /** The Preflight card: the latest verdict, or an invitation to run one. */
  renderPreflight(busy = false) {
    const report = this.store.preflight;
    this.$("[data-horizon]").classList.toggle("busy", busy);
    this.$("[data-preflight-when]").textContent = report ? Format.when(report.at) : "not run yet";
    if (busy) {
      this.fill("[data-preflight-summary]", html`<div class="verdict">Checking…</div><div class="verdict-sub">Reading every part of the machine fresh.</div>`);
      return;
    }
    if (!report) {
      this.fill("[data-preflight-summary]", html`<div class="verdict">Ready for a full check</div>
        <div class="verdict-sub">Preflight reads everything fresh and tests the connection. It takes a few seconds.</div>`);
      return;
    }
    const counts = { clear: 0, notice: 0, warning: 0, critical: 0 };
    report.checks.filter(c => c.applicable).forEach(c => { counts[c.severity] += 1; });
    const skipped = report.checks.filter(c => !c.applicable).length;
    const chips = ["critical", "warning", "notice", "clear"]
      .filter(k => counts[k])
      .map(k => html`<span class="chip t-${k}">${raw(Icons.severity[k])}${counts[k]} ${k}</span>`)
      .join("");
    const ran = report.checks.length - skipped;
    const sub = `${counts.clear} of ${ran} checks clear.` + (skipped ? ` ${skipped} don't apply to this machine.` : "");
    this.fill("[data-preflight-summary]", html`<div class="verdict">${report.verdict.title}</div>
      <div class="verdict-sub">${sub}</div><div class="counts">${raw(chips)}</div>`);
  }

  /** Redraws the trends. */
  repaint() {
    if (!this.visible) return;
    this.trends.forEach(t => t.draw(this.store.history));
  }
}
