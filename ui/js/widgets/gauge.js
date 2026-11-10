// A round instrument: a 270° ring that fills with the value, coloured by how close it is to
// its limits.

import { Dom } from "../dom.js";

const TAU = Math.PI * 2;
const SWEEP = 0.75;
const R = 52, C = 70;

export class Gauge {
  /** Builds a gauge. `limits` are the warning and critical values on a scale up to `max`. */
  constructor({ name, unit, max = 100, warn = Infinity, crit = Infinity }) {
    Object.assign(this, { name, unit, max, warn, crit });
    this.length = TAU * R * SWEEP;
    this.element = document.createElement("div");
    this.element.className = "gauge t-glass";
    this.element.innerHTML = this.markup();
    this.ring = this.element.querySelector(".ring");
    this.value = this.element.querySelector(".val");
    this.sub = this.element.querySelector(".sub");
  }

  /** The SVG and labels. */
  markup() {
    const ticks = [];
    for (let i = 0; i <= 10; i++) {
      const angle = Math.PI * 0.75 + Math.PI * 1.5 * (i / 10);
      const [x1, y1, x2, y2] = [62, 62, 66, 66].map((r, k) => (k % 2 ? C + Math.sin(angle) * r : C + Math.cos(angle) * r));
      const red = (i / 10) * this.max >= this.crit;
      ticks.push(`<line class="tick${red ? " red" : ""}" x1="${x1.toFixed(1)}" y1="${y1.toFixed(1)}" x2="${x2.toFixed(1)}" y2="${y2.toFixed(1)}"/>`);
    }
    const dash = `${this.length} ${TAU * R}`;
    return Dom.html`
      <svg viewBox="0 0 140 128" role="img" aria-label="${this.name}">
        ${Dom.raw(ticks.join(""))}
        <circle class="ring-bg" cx="${C}" cy="${C}" r="${R}" stroke-dasharray="${dash}" transform="rotate(135 ${C} ${C})"/>
        <circle class="ring" cx="${C}" cy="${C}" r="${R}" stroke-dasharray="${dash}" stroke-dashoffset="${this.length}" transform="rotate(135 ${C} ${C})"/>
        <text class="val" x="${C}" y="${C + 6}" text-anchor="middle">—</text>
        <text class="unit" x="${C}" y="${C + 22}" text-anchor="middle">${this.unit}</text>
      </svg>
      <span class="name">${this.name}</span>
      <span class="sub">&nbsp;</span>`;
  }

  /** Moves the ring and number to `value` (null shows a dash) with a line underneath. */
  set(value, sub = "") {
    const known = value !== null && value !== undefined && !Number.isNaN(value);
    const share = known ? Math.min(Math.max(value, 0), this.max) / this.max : 0;
    this.ring.style.strokeDashoffset = String(this.length * (1 - share));
    this.value.textContent = known ? Math.round(value) : "—";
    this.sub.textContent = sub || " ";
    this.element.className = `gauge ${this.tone(value, known)}`;
  }

  /** The colour for a value: glass while calm, warning and critical near the limits. */
  tone(value, known) {
    if (!known) return "t-faint";
    if (value >= this.crit) return "t-critical";
    if (value >= this.warn) return "t-warning";
    return "t-glass";
  }
}
