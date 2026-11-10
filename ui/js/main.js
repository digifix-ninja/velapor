// Velapor's window: creates the views and connects them to the Rust side.

import { Api } from "./api.js";
import { Dom } from "./dom.js";
import { Icons } from "./icons.js";
import { Logo } from "./logo.js";
import { Platform } from "./platform.js";
import { Store } from "./store.js";
import { Header } from "./views/header.js";
import { Tabs } from "./views/tabs.js";
import { Toast } from "./views/toast.js";
import { Sheet } from "./views/sheet.js";
import { OverviewView } from "./views/overview.js";
import { HardwareView } from "./views/hardware.js";
import { SoftwareView } from "./views/software.js";
import { NetworkView } from "./views/network.js";
import { PerformanceView } from "./views/performance.js";
import { IntegrityView } from "./views/integrity.js";
import { PreflightSheet } from "./views/preflight-sheet.js";
import { AboutPanel } from "./views/about-panel.js";

class App {
  /** Creates every part of the window. */
  constructor() {
    this.store = new Store();
    this.toast = new Toast(Dom.$("#toast"));
    this.header = new Header(this.store);
    this.views = {
      overview: new OverviewView(this.store),
      hardware: new HardwareView(this.store),
      software: new SoftwareView(this.store),
      network: new NetworkView(this.store, this.toast),
      performance: new PerformanceView(this.store),
      integrity: new IntegrityView(this.store),
    };
    this.tabs = new Tabs(this.store, { shown: key => this.views[key].render() });
    this.preflight = new PreflightSheet(this.store, this.toast, {
      started: () => this.preflightButton(true),
      finished: () => this.preflightButton(false),
    });
    this.about = new AboutPanel(this.toast);
    this.store.subscribe(reason => this.render(reason));
  }

  /** Paints the static parts, loads what the Rust side already knows, then listens. */
  async start() {
    Platform.markBody();
    Dom.$("#mark").innerHTML = Logo.html();
    for (const slot of Dom.$$("[data-icon]")) slot.outerHTML = Icons[slot.dataset.icon];
    this.bindControls();
    this.tabs.show(this.tabs.current);
    await Api.on("snapshot", snapshot => this.store.setSnapshot(snapshot));
    await Api.on("preflight", () => this.preflight.run());
    this.store.setHistory(await Api.getHistory().catch(() => []));
    this.store.setDisplays(await Api.getDisplays().catch(() => []));
    this.store.setSnapshot(await Api.getSnapshot().catch(() => null));
    setInterval(async () => this.store.setDisplays(await Api.getDisplays().catch(() => [])), 10_000);
  }

  /** Redraws the parts that change with every beat. Hidden tabs wait until they are shown. */
  render(reason) {
    this.header.render();
    this.tabs.render();
    const current = this.views[this.tabs.current];
    current.render();
    if (reason === "preflight" && current !== this.views.overview) this.views.overview.renderPreflight();
  }

  /** Buttons, keys and resizing. */
  bindControls() {
    Dom.$("#btn-preflight").addEventListener("click", () => this.preflight.run());
    Dom.$("#btn-about").addEventListener("click", () => this.about.show());
    Dom.$("#scrim").addEventListener("click", () => Sheet.closeAll());
    document.addEventListener("keydown", event => {
      if (event.key === "Escape") Sheet.closeAll();
    });
    const repaint = () => this.views[this.tabs.current].repaint();
    window.addEventListener("resize", repaint);
    document.addEventListener("themechange", repaint);
  }

  /** The Preflight button and the overview's horizon while a check runs. */
  preflightButton(busy) {
    const button = Dom.$("#btn-preflight");
    button.disabled = busy;
    Dom.$(".label", button).textContent = busy ? "Checking…" : "Run Preflight";
    this.views.overview.renderPreflight(busy);
  }
}

new App().start();
