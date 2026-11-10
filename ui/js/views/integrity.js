// The Integrity tab: security settings, and what should stay still since Velapor opened.

import { Dom } from "../dom.js";
import { Format } from "../format.js";
import { DomainView } from "./view.js";

const { html, raw } = Dom;
const POSTURE_TONE = { on: "t-clear", off: "t-warning", unknown: "t-faint" };
const POSTURE_WORD = { on: "On", off: "Off", unknown: "Couldn't be read" };

export class IntegrityView extends DomainView {
  /** Prepares the tab. */
  constructor(store) {
    super(store, "integrity");
  }

  /** Draws the tab. */
  render() {
    super.render();
    const snapshot = this.store.snapshot;
    if (!snapshot) return;
    const r = snapshot.readings;
    this.fill("[data-posture]", r.posture.length
      ? r.posture.map(p => html`<div><b>${p.name}</b><span class="status ${POSTURE_TONE[p.state]}">${POSTURE_WORD[p.state]}</span><small>${p.detail}</small></div>`).join("")
      : html`<p class="empty-note">This system doesn't report its security settings to Velapor.</p>`);
    this.$("[data-baseline-aside]").textContent = `compared with ${Format.when(snapshot.watchingSince)}`;
    this.fill("[data-baselines]", this.baselines(r));
  }

  /** One row per thing Velapor remembers from the start, and whether it changed. */
  baselines(r) {
    const changed = prefix => this.store.findings().filter(f => f.id.startsWith(prefix)).length;
    const list = r.fingerprints.map(f => [f.name, f.summary, changed(`integrity.changed:${f.key}`)]);
    list.push(["Startup items", Format.plural(r.software.startupItems.length, "item"), changed("integrity.new-startup-item:")]);
    list.push(["Listening ports", Format.plural(r.network.listening.length, "port"), changed("network.new-listener:")]);
    list.push(["Attached devices", Format.plural(r.devices.length, "device"), changed("hardware.new-device:")]);
    return list.map(([name, now, count]) => {
      const state = count
        ? html`<span class="status t-notice">${count === 1 ? "Changed" : `${count} new`}</span>`
        : html`<span class="status t-clear">Unchanged</span>`;
      return html`<tr><td class="proc">${name}</td><td class="mono dim breaks">${now}</td><td class="r">${raw(state)}</td></tr>`;
    }).join("");
  }
}
