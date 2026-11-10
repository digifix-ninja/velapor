// The Network tab: the speed test, traffic, open ports, interfaces and who is talking.

import { Api } from "../api.js";
import { Dom } from "../dom.js";
import { Format } from "../format.js";
import { Trend } from "../widgets/trend.js";
import { SpeedDial } from "../widgets/speed-dial.js";
import { DomainView, rows } from "./view.js";

const { html, raw } = Dom;

export class NetworkView extends DomainView {
  /** Builds the trends and connects the speed test button. */
  constructor(store, toast) {
    super(store, "network");
    this.toast = toast;
    this.trends = [
      new Trend({ name: "Download", unit: "Mb/s", pick: s => Format.megabits(s.received), show: v => v.toFixed(1) }),
      new Trend({ name: "Upload", unit: "Mb/s", pick: s => Format.megabits(s.sent), show: v => v.toFixed(1), color: "--flame" }),
    ];
    this.trends.forEach(t => this.$("[data-trends]").appendChild(t.element));
    this.dial = new SpeedDial(this.$(".speed-dial"));
    this.button = this.$("[data-run-speed]");
    this.button.addEventListener("click", () => this.runSpeedTest());
    Api.on("speed-progress", progress => this.progress(progress));
  }

  /** Draws the tab. */
  render() {
    super.render();
    const n = this.store.snapshot?.readings.network;
    if (!n) return;
    this.fill("[data-ports]", this.ports(n.listening));
    this.fill("[data-interfaces]", this.interfaces(n.interfaces));
    this.fill("[data-connections]", this.connections(n.busiest));
    this.$("[data-connections-aside]").textContent = `${Format.plural(n.established, "open connection")} in all`;
    this.repaint();
  }

  /** Programs waiting for connections. */
  ports(listening) {
    return rows(listening, 5, p => {
      const key = `${p.protocol}:${p.port}:${p.process ?? "?"}`;
      const isNew = this.store.hasFinding(`network.new-listener:${key}`);
      const reach = p.reachableFromNetwork
        ? html`<span class="tag ${isNew ? "new" : "glass"}">${isNew ? "NEW · " : ""}NETWORK</span>`
        : html`<span class="tag${isNew ? " new" : ""}">${isNew ? "NEW · " : ""}THIS MACHINE</span>`;
      return html`<tr><td class="mono num">${p.port}</td><td class="dim mono">${p.protocol.toLowerCase()}</td>
        <td class="proc">${p.process ?? "unknown"}</td><td class="mono dim">${p.address}</td><td class="r">${raw(reach)}</td></tr>`;
    }, "Nothing is waiting for connections.");
  }

  /** Interfaces with an address. */
  interfaces(interfaces) {
    const live = interfaces.filter(i => i.addresses.length);
    return rows(live, 4, i => html`<tr><td class="proc">${i.name}</td><td class="mono dim breaks">${i.addresses.join(", ")}</td>
      <td class="r mono num dim">${Format.rate(i.receivedPerSec)}</td><td class="r mono num dim">${Format.rate(i.sentPerSec)}</td></tr>`,
    "No interface has an address.");
  }

  /** The programs with the most open connections. */
  connections(busiest) {
    return rows(busiest, 3, c => html`<tr><td class="proc">${c.process}</td><td class="r mono num">${c.count}</td>
      <td class="mono dim breaks">${c.remotes.join(", ")}</td></tr>`, "No open connections.");
  }

  /** Runs the speed test and shows its result. */
  async runSpeedTest() {
    this.button.disabled = true;
    this.button.querySelector(".label").textContent = "Testing…";
    this.$("[data-speed-note]").textContent = "Testing against Cloudflare's nearest server. Only test data travels.";
    this.dial.reset("starting");
    try {
      const result = await Api.runSpeedTest();
      this.store.addSpeedTest(result);
      this.showResult(result);
    } catch (error) {
      this.dial.reset("no answer");
      this.$("[data-speed-note]").textContent = String(error);
      this.toast.error(error);
    } finally {
      this.button.disabled = false;
      this.button.querySelector(".label").textContent = "Run speed test";
    }
  }

  /** Moves the dial while the test runs. */
  progress(p) {
    if (p.phase === "latency") this.dial.latency(p.value);
    else this.dial.speed(p.value, p.phase === "download" ? "Mb/s down" : "Mb/s up");
  }

  /** Writes the four numbers and this session's earlier tests. */
  showResult(result) {
    this.dial.speed(result.downloadMbps, "Mb/s down");
    const fact = (key, value, digits) => { this.$(`[data-fact="${key}"]`).firstChild.textContent = value.toFixed(digits); };
    fact("down", result.downloadMbps, result.downloadMbps >= 100 ? 0 : 1);
    fact("up", result.uploadMbps, result.uploadMbps >= 100 ? 0 : 1);
    fact("ping", result.latencyMs, 0);
    fact("jitter", result.jitterMs, 1);
    this.$("[data-speed-note]").textContent = `Measured ${Format.when(Date.now())} against ${result.server}. Velapor keeps results only while it is open.`;
    const history = this.$("[data-speed-history]");
    history.hidden = this.store.speedTests.length < 2;
    history.querySelector("tbody").innerHTML = this.store.speedTests.map(t => html`<tr><td class="mono dim">${Format.when(t.at)}</td>
      <td class="r mono num">${t.downloadMbps.toFixed(0)}</td><td class="r mono num">${t.uploadMbps.toFixed(1)}</td>
      <td class="r mono num">${t.latencyMs.toFixed(0)}</td><td class="r mono num dim">${t.jitterMs.toFixed(1)}</td></tr>`).join("");
  }

  /** Redraws the trends. */
  repaint() {
    if (!this.visible) return;
    this.trends.forEach(t => t.draw(this.store.history));
  }
}
