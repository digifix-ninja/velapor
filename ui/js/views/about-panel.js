// About Velapor: version, promise, links, developer and licence.

import { Api } from "../api.js";
import { Dom } from "../dom.js";
import { Logo } from "../logo.js";
import { Icons } from "../icons.js";
import { Sheet } from "./sheet.js";

const { html, raw } = Dom;

export class AboutPanel extends Sheet {
  /** Takes over the sheet and opens links through the Rust side. */
  constructor(toast) {
    super(Dom.$("#about-sheet"));
    this.toast = toast;
    this.element.addEventListener("click", event => {
      const target = event.target.closest("[data-do], [data-url]");
      if (!target) return;
      if (target.dataset.do === "close") this.close();
      if (target.dataset.url) Api.openUrl(target.dataset.url).catch(error => this.toast.error(error));
    });
  }

  /** Fills the panel and slides it in. */
  async show() {
    const about = await Api.about().catch(() => null);
    if (!about) return;
    this.element.innerHTML = html`
      <header>
        <span class="eyebrow"><span>About</span></span>
        <button class="btn icon-btn" data-do="close" title="Close" type="button">${raw(Icons.close)}</button>
      </header>
      <div class="scroll"><div class="about">
        ${raw(Logo.html("The Velapor mark: a gauge whose needles form a V, with a candle flame in the dial"))}
        <h2>VELAPOR</h2>
        <span class="version">Version ${about.version}</span>
        <p>Vela por ti: a candle that keeps watch over your machine. Velapor reads, explains what it sees, and warns you when something needs you.</p>
        <div class="promise">
          <span><b>Read-only.</b> Velapor never changes a file, a setting or a process.</span>
          <span><b>Stores nothing.</b> What it sees lives in memory and is gone when you quit.</span>
          <span><b>Stays here.</b> Nothing about your machine leaves it. Only the speed test and Preflight's connection check use the internet, and only when you ask.</span>
        </div>
        <div class="links">
          <button class="btn" data-url="${about.appWebsite}" type="button">velapor.app</button>
          ${raw(about.sourceCode ? html`<button class="btn" data-url="${about.sourceCode}" type="button">Source code</button>` : "")}
          <button class="btn" data-url="${about.family}" type="button">DigiFix.Ninja</button>
        </div>
        <p class="legal">© ${about.copyrightYear} ${about.developer} · ${about.licence} licence</p>
      </div></div>`;
    this.open();
  }
}
