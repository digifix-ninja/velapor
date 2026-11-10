// Preflight: a full check on demand. The sheet lists every check as it comes back, then the
// findings, like a pilot's checklist before take-off.

import { Api } from "../api.js";
import { Dom } from "../dom.js";
import { Format } from "../format.js";
import { Icons, SEVERITY_WORD } from "../icons.js";
import { Sheet } from "./sheet.js";
import { FindingList } from "./finding-list.js";
import { DOMAINS } from "../store.js";

const { html, raw } = Dom;

export class PreflightSheet extends Sheet {
  /** Takes over the sheet; `started` and `finished` let the overview follow along. */
  constructor(store, toast, { started, finished }) {
    super(Dom.$("#preflight-sheet"));
    Object.assign(this, { store, toast, started, finished });
    this.running = false;
    this.element.addEventListener("click", event => {
      if (event.target.closest("[data-do=close]")) this.close();
      if (event.target.closest("[data-do=again]")) this.run();
    });
  }

  /** Runs Preflight and shows the result. */
  async run() {
    if (this.running) return;
    this.running = true;
    this.started();
    this.paint(html`<p class="empty-note">Reading every part of the machine fresh and testing the connection…</p>`);
    this.open();
    try {
      const report = await Api.runPreflight();
      this.store.setPreflight(report);
      this.paint(this.report(report));
      new FindingList(Dom.$("[data-findings]", this.element)).render(report.findings, "Nothing to report. Cleared for take-off.");
    } catch (error) {
      this.paint(html`<p class="empty-note">${String(error)}</p>`);
      this.toast.error(error);
    } finally {
      this.running = false;
      this.finished();
    }
  }

  /** Fills the sheet with a header and `body`. */
  paint(body) {
    this.element.innerHTML = html`
      <header>
        <span class="eyebrow"><span>Preflight</span></span>
        <button class="btn icon-btn" data-do="close" title="Close" type="button">${raw(Icons.close)}</button>
      </header>
      <div class="scroll">${raw(body)}</div>`;
  }

  /** The verdict, the checklist and the findings. */
  report(report) {
    const connection = [
      { name: "Name lookup", domain: "network", ok: report.connection.names.ok, detail: `${report.connection.names.detail} · ${report.connection.names.millis} ms` },
      { name: "Internet", domain: "network", ok: report.connection.internet.ok, detail: `${report.connection.internet.detail} · ${report.connection.internet.millis} ms` },
    ];
    const checks = report.checks.map((c, i) => {
      const severity = c.applicable ? c.severity : "unknown";
      const word = c.applicable ? SEVERITY_WORD[c.severity] : "n/a";
      const area = DOMAINS.find(d => d.key === c.domain)?.label ?? c.domain;
      return this.row(i, severity, area, c.name, word);
    });
    connection.forEach((c, i) => checks.push(this.row(report.checks.length + i, c.ok ? "clear" : "warning", "Network", `${c.name}`, c.ok ? "Clear" : "Failed", c.detail)));
    return html`
      <div class="t-${report.verdict.severity}">
        <div class="verdict">${report.verdict.title}</div>
        <div class="verdict-sub">${report.verdict.detail} Took ${(report.tookMs / 1000).toFixed(1)} s · ${Format.when(report.at)}.</div>
      </div>
      <section class="card"><div class="eyebrow"><span>Checklist</span><span class="aside">${Format.plural(checks.length, "check")}</span></div>
        <div class="pf-list">${raw(checks.join(""))}</div></section>
      <section class="card"><div class="eyebrow"><span>Findings</span></div><div class="findings" data-findings></div></section>
      <button class="btn" data-do="again" type="button">${raw(Icons.preflight)}<span>Run again</span></button>`;
  }

  /** One line of the checklist, appearing a moment after the one before it. */
  row(index, severity, area, name, word, detail = "") {
    const tone = severity === "unknown" ? "t-faint" : `t-${severity}`;
    return html`<div class="pf-row ${tone}" style="animation-delay:${index * 45}ms">
      <span class="sev">${raw(Icons.severity[severity])}</span>
      <span class="what"><small>${area}</small>${name}${raw(detail ? html` <span class="dim">· ${detail}</span>` : "")}</span>
      <span class="word">${word}</span></div>`;
  }
}
