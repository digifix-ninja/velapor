// Velapor's mark: a gauge whose two needles form a V, with a candle flame in the dial.

const TICKS = [
  [6.89, 36.43, 11.32, 35.65, 2.2], [6.67, 29.04, 9.35, 29.35, 1.4], [8.59, 21.9, 11.06, 22.97, 1.4],
  [12.47, 15.61, 15.91, 18.5, 2.2], [17.99, 10.7, 19.47, 12.95, 1.4], [24.69, 7.57, 25.46, 10.16, 1.4],
  [32, 6.5, 32, 11, 2.2], [39.31, 7.57, 38.54, 10.16, 1.4], [46.01, 10.7, 44.53, 12.95, 1.4],
  [51.53, 15.61, 48.09, 18.5, 2.2], [55.41, 21.9, 52.94, 22.97, 1.4], [57.33, 29.04, 54.65, 29.35, 1.4],
  [57.11, 36.43, 52.68, 35.65, 2.2],
];

export class Logo {
  /** The mark as inline SVG, in the text colour with a living flame. */
  static html(label = "") {
    const ticks = TICKS.map(([x1, y1, x2, y2, w]) => `<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke-width="${w}"/>`).join("");
    const aria = label ? `role="img" aria-label="${label}"` : 'aria-hidden="true"';
    return `<svg class="mark" viewBox="0 0 64 64" ${aria}>
      <circle cx="32" cy="32" r="29" fill="none" stroke="currentColor" stroke-width="2.6"/>
      <g stroke="currentColor" stroke-linecap="round">${ticks}</g>
      <g fill="currentColor"><path d="M34 43.68 13.2 16.6 30 46.32Z"/><path d="M34 46.32 50.8 16.6 30 43.68Z"/></g>
      <circle cx="32" cy="45" r="3.6" fill="currentColor"/>
      <path class="flame" fill="var(--flame)" d="M32 35C27.6 35 25.4 31.6 26 28c.6-3.6 3.4-5.4 4-9.2.2-1.4 0-2.6-.4-3.8 3.8 1.8 7.4 5.6 8.2 10 .8 5-1.6 10-5.8 10Z"/>
      <path class="flame" fill="#fff1d6" d="M32 32.6c-1.8 0-2.8-1.4-2.6-3 .2-1.6 1.4-2.6 1.8-4.6 1.8 1.2 3.4 3 3.4 4.8 0 1.6-1 2.8-2.6 2.8Z"/>
    </svg>`;
  }
}
