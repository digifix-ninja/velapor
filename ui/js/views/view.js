// The base of every tab: finds its section in the page and fills its parts.

import { Dom } from "../dom.js";
import { FindingList } from "./finding-list.js";
import { CheckList } from "./check-list.js";

export class View {
  /** Takes over `<section id="view-{key}">`. */
  constructor(store, key) {
    this.store = store;
    this.key = key;
    this.root = Dom.$(`#view-${key}`);
  }

  /** The first element inside this tab matching `selector`. */
  $(selector) {
    return Dom.$(selector, this.root);
  }

  /** Replaces the inside of `selector` with `html`, if it changed. */
  fill(selector, html) {
    const element = this.$(selector);
    if (element && element.innerHTML !== html) element.innerHTML = html;
  }

  /** Whether the tab is on screen. */
  get visible() {
    return !this.root.hidden;
  }

  /** Draws the tab. Each tab fills in its own parts. */
  render() {}

  /** Redraws canvases after the tab becomes visible or the window resizes. */
  repaint() {}
}

/** A tab for one of the five watched areas: it always has findings and a list of checks. */
export class DomainView extends View {
  /** Adds the findings and checks to the tab. */
  constructor(store, key) {
    super(store, key);
    this.findingList = new FindingList(this.$("[data-findings]"));
    this.checkList = new CheckList(this.$("[data-checks]"), this.$("[data-checks-aside]"));
  }

  /** Draws the findings and checks of this area. */
  render() {
    this.findingList.render(this.store.findings(this.key), "Nothing to report here.");
    this.checkList.render(this.store.checks(this.key));
  }
}

/** Builds table rows, or one row with a note when there is nothing to show. */
export function rows(items, columns, toRow, empty) {
  if (!items.length) return Dom.html`<tr><td colspan="${columns}" class="dim">${empty}</td></tr>`;
  return items.map(toRow).join("");
}

/** Builds a key and value list from pairs, skipping pairs without a value. */
export function pairs(list) {
  return list
    .filter(([, value]) => value !== null && value !== undefined && value !== "")
    .map(([key, value]) => Dom.html`<dt>${key}</dt><dd>${value}</dd>`)
    .join("");
}

/** A usage bar for table cells, coloured by severity once it gets high. */
export function bar(percent, warn = 85, crit = 95) {
  const tone = percent >= crit ? "t-critical" : percent >= warn ? "t-warning" : "t-glass";
  return Dom.html`<span class="bar ${tone}"><i style="width:${Math.min(100, Math.max(0, percent)).toFixed(0)}%"></i></span>`;
}
