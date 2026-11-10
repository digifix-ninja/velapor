// The Hardware tab: what the machine is, what is attached, power, drives and sensors.

import { Dom } from "../dom.js";
import { Format } from "../format.js";
import { Gauge } from "../widgets/gauge.js";
import { DomainView, rows, pairs, bar } from "./view.js";
import { Instruments } from "./instruments.js";

const { html, raw } = Dom;

export class HardwareView extends DomainView {
  /** Builds the gauges. */
  constructor(store) {
    super(store, "hardware");
    this.gauges = {
      cpu: new Gauge({ name: "Processor", unit: "%", warn: 85, crit: 95 }),
      memory: new Gauge({ name: "Memory", unit: "%", warn: 90, crit: 97 }),
      heat: new Gauge({ name: "Temperature", unit: "°C", max: 110, warn: 85, crit: 95 }),
      battery: new Gauge({ name: "Battery", unit: "%" }),
    };
    Object.values(this.gauges).forEach(g => this.$("[data-gauges]").appendChild(g.element));
  }

  /** Draws the tab. */
  render() {
    super.render();
    const r = this.store.snapshot?.readings;
    if (!r) return;
    Instruments.setCpu(this.gauges.cpu, r);
    Instruments.setMemory(this.gauges.memory, r);
    Instruments.setHeat(this.gauges.heat, r);
    Instruments.setBattery(this.gauges.battery, r);
    this.fill("[data-machine]", this.machine(r.machine));
    this.fill("[data-power]", this.power(r));
    this.fill("[data-devices]", this.devices(r.devices));
    this.fill("[data-drives]", this.drives(r.disks));
    this.fill("[data-sensors]", this.sensors(r.sensors));
  }

  /** What is inside. */
  machine(m) {
    const model = [m.maker, m.model].filter(Boolean).join(" ") || null;
    const cores = m.physicalCores && m.physicalCores !== m.cores ? `${m.physicalCores} cores · ${m.cores} threads` : Format.plural(m.cores, "core");
    return pairs([
      ["Model", model],
      ["Model id", m.modelId],
      ["Processor", m.processor],
      ["Cores", cores],
      ["Memory", Format.bytes(m.memory)],
      ["Graphics", m.graphics.join(", ")],
      ["Architecture", m.architecture],
      ["Name", m.hostName],
    ]);
  }

  /** The adapter and the battery. */
  power(r) {
    const adapter = r.devices.find(d => d.kind === "Power");
    const b = r.battery;
    const list = [["Adapter", adapter ? `${adapter.name}${adapter.detail ? ` · ${adapter.detail}` : ""}` : "not connected"]];
    if (b) {
      list.push(
        ["Battery", `${b.state} · ${Format.percent(b.charge)}`],
        ["Health", `${Format.percent(b.health)} of its original capacity`],
        ["Charge cycles", b.cycles ?? null],
        ["Temperature", b.temperature ? `${b.temperature.toFixed(0)} °C` : null],
        ["Kind", [b.technology, b.model].filter(Boolean).join(" · ")],
      );
    } else {
      list.push(["Battery", "none"]);
    }
    return pairs(list);
  }

  /** Every attached device, the screens included. */
  devices(devices) {
    const all = [...this.store.displays, ...devices.filter(d => d.kind !== "Power")];
    return rows(all, 4, d => {
      const isNew = this.store.hasFinding(`hardware.new-device:${d.key}`);
      const state = isNew ? html`<span class="tag new">NEW</span>` : html`<span class="tag glass">${d.kind}</span>`;
      const maker = d.maker ? html` <span class="dim">· ${d.maker}</span>` : "";
      return html`<tr><td class="proc">${d.name}${raw(maker)}</td><td class="dim">${d.connection}</td>
        <td class="mono dim">${d.detail}</td><td class="r">${raw(state)}</td></tr>`;
    }, "Nothing attached that Velapor can see.");
  }

  /** Space and activity per drive. */
  drives(disks) {
    return rows(disks, 5, d => {
      const used = d.total ? ((d.total - d.available) / d.total) * 100 : 0;
      return html`<tr><td class="proc">${d.mountPoint} <span class="dim">· ${d.kind} · ${d.fileSystem}</span></td>
        <td class="mono num">${raw(bar(used, 90, 97))}${Format.percent(used)}</td>
        <td class="r mono num dim">${Format.bytes(d.available)} of ${Format.bytes(d.total)}</td>
        <td class="r mono num dim">${Format.rate(d.readPerSec)}</td><td class="r mono num dim">${Format.rate(d.writtenPerSec)}</td></tr>`;
    }, "No drive answers.");
  }

  /** Temperatures, each placed in its safe range. */
  sensors(sensors) {
    return rows(sensors, 3, s => {
      const limit = s.critical && s.critical > 40 ? s.critical : 95;
      const top = limit + 10;
      const safe = ((limit - 10) / top) * 100;
      const now = Math.min(100, (s.celsius / top) * 100);
      const tone = s.celsius >= limit ? "t-critical" : s.celsius >= limit - 10 ? "t-warning" : "t-glass";
      return html`<tr><td class="proc">${s.label}</td><td class="r mono num">${s.celsius.toFixed(0)} °C</td>
        <td><span class="range ${tone}"><span class="band" style="width:${safe.toFixed(0)}%"></span><span class="now" style="left:${now.toFixed(0)}%"></span></span></td></tr>`;
    }, "No temperature sensor answers on this machine.");
  }
}
