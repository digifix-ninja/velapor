// The checks of one tab, so the person sees what Velapor looked at, not only what it found.

import { Dom } from "../dom.js";
import { Format } from "../format.js";
import { SEVERITY_WORD } from "../icons.js";

export class CheckList {
  /** Takes over `host`, and `aside` for the count. */
  constructor(host, aside) {
    this.host = host;
    this.aside = aside;
  }

  /** Draws every check with its outcome. */
  render(checks) {
    const markup = checks
      .map(c => {
        const tone = c.applicable ? `t-${c.severity}` : "t-faint";
        const word = c.applicable ? SEVERITY_WORD[c.severity] : "not on this machine";
        return Dom.html`<span class="status ${tone}" title="${word}">${c.name}</span>`;
      })
      .join("");
    if (this.host.innerHTML !== markup) this.host.innerHTML = markup;
    if (this.aside) {
      const ran = checks.filter(c => c.applicable).length;
      this.aside.textContent = `${Format.plural(ran, "check")} on this machine`;
    }
  }
}
