// The Software tab: the operating system and what starts with it.

import { Dom } from "../dom.js";
import { Format } from "../format.js";
import { DomainView, rows, pairs } from "./view.js";

const { html, raw } = Dom;

export class SoftwareView extends DomainView {
  /** Prepares the tab. */
  constructor(store) {
    super(store, "software");
  }

  /** Draws the tab. */
  render() {
    super.render();
    const s = this.store.snapshot?.readings.software;
    if (!s) return;
    this.fill("[data-system]", pairs([
      ["System", `${s.system} ${s.version}`.trim()],
      ["Kernel", s.kernel],
      ["Name", s.hostName],
      ["Running for", Format.duration(s.uptime)],
      ["Started", s.bootTime ? Format.when(s.bootTime * 1000) : null],
    ]));
    this.$("[data-startup-aside]").textContent = `${Format.plural(s.startupItems.length, "item")} start with the system`;
    this.fill("[data-startup]", rows(s.startupItems, 4, item => {
      const key = `${item.kind}|${item.location}|${item.name}`;
      const isNew = this.store.hasFinding(`integrity.new-startup-item:${key}`);
      const state = isNew ? html`<span class="tag new">NEW</span>` : html`<span class="status t-clear">Known</span>`;
      return html`<tr><td class="proc">${item.name}</td><td class="dim">${item.kind}</td>
        <td class="mono dim breaks">${item.location}</td><td class="r">${raw(state)}</td></tr>`;
    }, "Nothing is set to start with the system."));
  }
}
