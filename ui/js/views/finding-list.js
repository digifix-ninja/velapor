// A list of findings. Each explains itself: what it means, why, what to do, and the evidence.
// Opened explanations stay open across updates.

import { Dom } from "../dom.js";
import { Format } from "../format.js";
import { Icons, SEVERITY_WORD } from "../icons.js";

const { html, raw } = Dom;

export class FindingList {
  /** Takes over `host`. */
  constructor(host) {
    this.host = host;
    this.open = new Set();
    host.addEventListener("toggle", event => {
      const id = event.target.closest("[data-id]")?.dataset.id;
      if (!id) return;
      if (event.target.open) this.open.add(id); else this.open.delete(id);
    }, true);
  }

  /** Draws `findings`, or `empty` when there are none. */
  render(findings, empty) {
    const markup = findings.length
      ? findings.map(f => this.finding(f)).join("")
      : html`<p class="empty-note">${empty}</p>`;
    if (this.host.innerHTML !== markup) this.host.innerHTML = markup;
  }

  /** The markup of one finding. */
  finding(f) {
    const cause = f.likelyCause ? html`<dt>Likely cause</dt><dd>${f.likelyCause}</dd>` : "";
    const evidence = f.evidence.length
      ? html`<dt>Evidence</dt><dd><div class="evidence selectable">${raw(f.evidence.map(e => html`<span>${e.label}</span><span>${e.value}</span>`).join(""))}</div></dd>`
      : "";
    const open = this.open.has(f.id) ? " open" : "";
    return html`<article class="finding t-${f.severity}" data-id="${f.id}">
      <div class="f-head">
        <h3><span class="sev" title="${SEVERITY_WORD[f.severity]}">${raw(Icons.severity[f.severity])}</span><span>${f.title}</span></h3>
        <span class="when">${SEVERITY_WORD[f.severity]} · ${Format.since(f.since)}</span>
      </div>
      <p>${f.explanation}</p>
      <details${raw(open)}><summary>Why, and what to do</summary>
        <dl>${raw(cause)}<dt>What to do</dt><dd>${f.recommendation}</dd>${raw(evidence)}</dl>
      </details>
    </article>`;
  }
}
