// Gauge readings and trends that more than one tab shows, written once.

import { Format } from "../format.js";
import { Trend } from "../widgets/trend.js";

export class Instruments {
  /** Processor load, with the core count underneath. */
  static setCpu(gauge, r) {
    const cpu = r.processor.cpu;
    const ghz = cpu.frequencyMhz ? ` · ${(cpu.frequencyMhz / 1000).toFixed(1)} GHz` : "";
    gauge.set(cpu.usage, `${Format.plural(cpu.cores, "core")}${ghz}`);
  }

  /** Memory in use, with the amounts underneath. */
  static setMemory(gauge, r) {
    const m = r.processor.memory;
    gauge.set(m.total ? (m.used / m.total) * 100 : null, `${Format.bytes(m.used)} of ${Format.bytes(m.total)}`);
  }

  /** Swap in use. */
  static setSwap(gauge, r) {
    const m = r.processor.memory;
    if (!m.swapTotal) return gauge.set(null, "no swap set up");
    gauge.set((m.swapUsed / m.swapTotal) * 100, `${Format.bytes(m.swapUsed)} of ${Format.bytes(m.swapTotal)}`);
  }

  /** The warmest sensor. */
  static setHeat(gauge, r) {
    if (!r.sensors.length) return gauge.set(null, "no sensor answers");
    const hottest = r.sensors.reduce((a, b) => (b.celsius > a.celsius ? b : a));
    gauge.set(hottest.celsius, hottest.label);
  }

  /** The fullest drive that can be written to. */
  static setFullestDisk(gauge, r) {
    const disks = r.disks.filter(d => d.total && !d.readOnly);
    if (!disks.length) return gauge.set(null, "no drive answers");
    const used = d => ((d.total - d.available) / d.total) * 100;
    const fullest = disks.reduce((a, b) => (used(b) > used(a) ? b : a));
    gauge.set(used(fullest), `${Format.bytes(fullest.available)} free · ${fullest.mountPoint}`);
  }

  /** The battery's charge, or the warmest sensor when there is no battery. */
  static setBattery(gauge, r) {
    const b = r.battery;
    if (!b) return gauge.set(null, "no battery");
    gauge.set(b.charge, `${b.state} · health ${Format.percent(b.health)}`);
  }

  /** The processor load over time. */
  static cpuTrend() {
    return new Trend({ name: "Processor", unit: "%", pick: s => s.cpu, max: 100 });
  }

  /** Memory use over time. */
  static memoryTrend() {
    return new Trend({ name: "Memory", unit: "%", pick: s => s.memory, max: 100 });
  }
}
