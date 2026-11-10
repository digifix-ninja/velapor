// The Performance tab: what keeps the machine busy, now and over the last hour.

import { Dom } from "../dom.js";
import { Format } from "../format.js";
import { Trend } from "../widgets/trend.js";
import { DomainView, rows, bar } from "./view.js";
import { Instruments } from "./instruments.js";

const { html, raw } = Dom;

export class PerformanceView extends DomainView {
  /** Builds the trends. */
  constructor(store) {
    super(store, "performance");
    this.trends = [
      Instruments.cpuTrend(),
      Instruments.memoryTrend(),
      new Trend({ name: "Drive reads", unit: "MB/s", pick: s => s.diskRead / 1e6, show: v => v.toFixed(1) }),
      new Trend({ name: "Drive writes", unit: "MB/s", pick: s => s.diskWritten / 1e6, show: v => v.toFixed(1), color: "--flame" }),
    ];
    this.trends.forEach(t => this.$("[data-trends]").appendChild(t.element));
  }

  /** Draws the tab. */
  render() {
    super.render();
    const r = this.store.snapshot?.readings;
    if (!r) return;
    const load = r.processor.cpu.loadAverage;
    this.$("[data-load-aside]").textContent = load
      ? `load ${load.map(v => v.toFixed(2)).join(" · ")} · ${Format.plural(r.performance.processCount, "process", "processes")}`
      : Format.plural(r.performance.processCount, "process", "processes");
    this.fill("[data-by-cpu]", rows(r.performance.byCpu, 5, p => html`<tr><td class="proc">${p.name}</td>
      <td class="mono num">${raw(bar(p.cpu, 60, 90))}${p.cpu.toFixed(1)} %</td>
      <td class="r mono num dim">${Format.bytes(p.memory)}</td>
      <td class="r mono num dim">${Format.rate(p.readPerSec + p.writtenPerSec)}</td>
      <td class="r mono dim">${Format.duration(p.runTime)}</td></tr>`, "No process answers."));
    const total = r.processor.memory.total || 1;
    this.fill("[data-by-memory]", rows(r.performance.byMemory, 4, p => html`<tr><td class="proc">${p.name}</td>
      <td class="mono num">${raw(bar((p.memory / total) * 100, 30, 50))}${Format.bytes(p.memory)}</td>
      <td class="r mono num dim">${p.cpu.toFixed(1)} %</td><td class="r mono dim">${Format.duration(p.runTime)}</td></tr>`, "No process answers."));
    this.repaint();
  }

  /** Redraws the trends. */
  repaint() {
    if (!this.visible) return;
    this.trends.forEach(t => t.draw(this.store.history));
  }
}
