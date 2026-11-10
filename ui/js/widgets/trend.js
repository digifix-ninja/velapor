// A small line chart of the last hour, drawn on a canvas.

import { Dom } from "../dom.js";
import { Theme } from "../theme.js";

const TAU = Math.PI * 2;

export class Trend {
  /** Builds a trend for one value of the history. `pick(sample)` returns the number,
   *  `show(value)` the text next to the name, `max` the top of the scale (or "auto"). */
  constructor({ name, unit, pick, show = v => v.toFixed(0), max = "auto", color = "--glass" }) {
    Object.assign(this, { name, unit, pick, show, max, color });
    this.element = document.createElement("div");
    this.element.className = "trend";
    this.element.innerHTML = Dom.html`
      <header><span class="t-name">${name}</span><span class="t-now"><span class="v">—</span><small>${unit}</small></span></header>
      <canvas aria-hidden="true"></canvas>
      <div class="t-foot"><span class="from">—</span><span>now</span></div>`;
    this.canvas = this.element.querySelector("canvas");
    this.now = this.element.querySelector(".v");
    this.from = this.element.querySelector(".from");
  }

  /** Redraws the chart from the history (oldest first). */
  draw(history) {
    const points = history.map(this.pick).map(v => (v === null || v === undefined || Number.isNaN(v) ? null : v));
    const known = points.filter(v => v !== null);
    this.now.textContent = known.length ? this.show(known[known.length - 1]) : "—";
    if (history.length) this.from.textContent = `−${Math.max(1, Math.round((history[history.length - 1].at - history[0].at) / 60000))} min`;
    this.paint(points, known);
  }

  /** Draws the line, its soft fill and the emphasised last point. */
  paint(points, known) {
    const dpr = window.devicePixelRatio || 1;
    const w = this.canvas.clientWidth, h = this.canvas.clientHeight;
    if (!w || !h) return; // hidden tab: drawn when it is shown
    this.canvas.width = w * dpr;
    this.canvas.height = h * dpr;
    const ctx = this.canvas.getContext("2d");
    ctx.scale(dpr, dpr);
    ctx.strokeStyle = Theme.token("--line");
    ctx.lineWidth = 1;
    for (let i = 1; i < 4; i++) {
      const y = Math.round((h * i) / 4) + 0.5;
      ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(w, y); ctx.stroke();
    }
    if (known.length < 2) return;
    const top = this.max === "auto" ? Math.max(...known) * 1.25 || 1 : this.max;
    const pad = 4, n = points.length;
    const x = i => (i / (n - 1)) * (w - pad * 2) + pad;
    const y = v => h - pad - (Math.min(v, top) / top) * (h - pad * 2);
    const color = Theme.token(this.color);
    ctx.beginPath();
    let started = false, first = 0, last = 0;
    points.forEach((v, i) => {
      if (v === null) return;
      if (!started) { ctx.moveTo(x(i), y(v)); started = true; first = i; } else ctx.lineTo(x(i), y(v));
      last = i;
    });
    ctx.lineWidth = 1.5; ctx.strokeStyle = color; ctx.lineJoin = "round"; ctx.stroke();
    ctx.lineTo(x(last), h); ctx.lineTo(x(first), h); ctx.closePath();
    const fill = ctx.createLinearGradient(0, 0, 0, h);
    fill.addColorStop(0, color + "40"); fill.addColorStop(1, color + "00");
    ctx.fillStyle = fill; ctx.fill();
    const lx = x(last), ly = y(points[last]);
    ctx.beginPath(); ctx.arc(lx, ly, 3, 0, TAU); ctx.fillStyle = color; ctx.fill();
    ctx.beginPath(); ctx.arc(lx, ly, 7, 0, TAU); ctx.fillStyle = color + "30"; ctx.fill();
  }
}
