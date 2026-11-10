// The title bar's verdict: a pulsing dot in the colour of the worst finding, and one line.

import { Dom } from "../dom.js";
import { Format } from "../format.js";

export class Header {
  /** Takes over the headline and the footer's "watching since". */
  constructor(store) {
    this.store = store;
    this.headline = Dom.$("#headline");
    this.watching = Dom.$("[data-watching]");
  }

  /** Draws the verdict. */
  render() {
    const snapshot = this.store.snapshot;
    if (!snapshot) return;
    const { headline } = snapshot;
    this.headline.className = `headline t-${headline.severity}`;
    Dom.$(".title", this.headline).textContent = headline.title + ".";
    Dom.$(".detail", this.headline).textContent = headline.detail;
    const checks = snapshot.checks.filter(c => c.applicable).length;
    Dom.$(".meta", this.headline).textContent = `· ${Format.plural(checks, "check")} · every 2 s`;
    const host = snapshot.readings.machine.hostName || "this machine";
    this.watching.textContent = `Watching ${host} since ${Format.when(snapshot.watchingSince)}`;
  }
}
