// The strip of tiles at the top: the overview and the five watched areas. Each tile shows
// how its area is doing and opens its tab.

import { Dom } from "../dom.js";
import { DOMAINS } from "../store.js";
import { Format } from "../format.js";
import { SEVERITY_WORD } from "../icons.js";

const { html } = Dom;
const KEYS = ["overview", ...DOMAINS.map(d => d.key)];

export class Tabs {
  /** Builds the tiles and follows the address (#hardware, #network…). */
  constructor(store, { shown }) {
    this.store = store;
    this.shown = shown;
    this.strip = Dom.$("#tabs");
    this.strip.innerHTML = KEYS.map(key => html`
      <button class="domain t-faint" role="tab" data-view="${key}" type="button">
        <span class="label"><i></i>${Tabs.label(key)}</span>
        <span class="state">—</span><span class="meta">&nbsp;</span>
      </button>`).join("");
    this.strip.addEventListener("click", event => {
      const tile = event.target.closest("[data-view]");
      if (tile) location.hash = tile.dataset.view;
    });
    window.addEventListener("hashchange", () => this.show(location.hash.slice(1)));
  }

  /** The name on a tile. */
  static label(key) {
    return key === "overview" ? "Overview" : DOMAINS.find(d => d.key === key).label;
  }

  /** The tab on screen. */
  get current() {
    const key = location.hash.slice(1);
    return KEYS.includes(key) ? key : "overview";
  }

  /** Shows one tab and marks its tile. */
  show(key) {
    if (!KEYS.includes(key)) key = "overview";
    for (const k of KEYS) Dom.$(`#view-${k}`).hidden = k !== key;
    for (const tile of Dom.$$(".domain", this.strip)) {
      const on = tile.dataset.view === key;
      tile.classList.toggle("on", on);
      tile.setAttribute("aria-selected", String(on));
    }
    this.shown(key);
  }

  /** Updates every tile from the latest snapshot. */
  render() {
    const snapshot = this.store.snapshot;
    if (!snapshot) return;
    for (const tile of Dom.$$(".domain", this.strip)) {
      const { state, meta, severity } = this.describe(tile.dataset.view, snapshot);
      tile.className = `domain t-${severity}${tile.dataset.view === this.current ? " on" : ""}`;
      Dom.$(".state", tile).textContent = state;
      Dom.$(".meta", tile).textContent = meta;
    }
  }

  /** What one tile says. */
  describe(key, snapshot) {
    if (key === "overview") {
      const count = snapshot.findings.length;
      return {
        severity: snapshot.headline.severity,
        state: count ? Format.plural(count, "finding") : "All quiet",
        meta: `${Format.plural(snapshot.checks.filter(c => c.applicable).length, "check")} running`,
      };
    }
    const summary = snapshot.domains.find(d => d.domain === key);
    let state = `${summary.findings} findings`;
    if (summary.findings === 0) state = "Clear";
    if (summary.findings === 1) state = `1 ${SEVERITY_WORD[summary.severity].toLowerCase()}`;
    return { severity: summary.severity, state, meta: this.meta(key, snapshot.readings) };
  }

  /** The small line under each tile: the reading people glance at. */
  meta(key, r) {
    switch (key) {
      case "hardware": {
        const temperatures = r.sensors.map(s => s.celsius);
        const second = temperatures.length
          ? `${Math.max(...temperatures).toFixed(0)} °C`
          : Format.plural(r.devices.length, "device");
        return `CPU ${Format.percent(r.processor.cpu.usage)} · ${second}`;
      }
      case "software":
        return `${r.software.system} ${r.software.version}`.trim() || "—";
      case "network":
        return `↓ ${Format.rate(r.network.receivedPerSec)} · ${Format.plural(r.network.listening.length, "port")}`;
      case "performance":
        return Format.plural(r.performance.processCount, "process", "processes");
      case "integrity": {
        const on = r.posture.filter(p => p.state === "on").length;
        return r.posture.length ? `${on} of ${r.posture.length} protections on` : "—";
      }
      default:
        return "";
    }
  }
}
