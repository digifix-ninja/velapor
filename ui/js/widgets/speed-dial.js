// The half-dial of the speed test, on a logarithmic scale that reads well from 5 to 1 000 Mb/s.

const LENGTH = 251.3;
const FULL_SCALE = 1000;

export class SpeedDial {
  /** Takes over the dial inside `root`. */
  constructor(root) {
    this.ring = root.querySelector(".sd-ring");
    this.value = root.querySelector(".sd-val");
    this.unit = root.querySelector(".sd-unit");
  }

  /** Shows a speed in Mb/s with a label underneath. */
  speed(mbps, label) {
    const share = Math.log10(1 + Math.max(0, mbps)) / Math.log10(1 + FULL_SCALE);
    this.ring.style.strokeDashoffset = String(LENGTH * (1 - Math.min(1, share)));
    this.value.textContent = mbps >= 100 ? Math.round(mbps) : mbps.toFixed(1);
    this.unit.textContent = label;
  }

  /** Shows a round-trip time in milliseconds (the ring stays low: lower is better). */
  latency(ms) {
    this.ring.style.strokeDashoffset = String(LENGTH * (1 - Math.min(1, ms / 200) * 0.5));
    this.value.textContent = Math.round(ms);
    this.unit.textContent = "ms ping";
  }

  /** Empties the dial. */
  reset(label = "press to measure") {
    this.ring.style.strokeDashoffset = String(LENGTH);
    this.value.textContent = "—";
    this.unit.textContent = label;
  }
}
