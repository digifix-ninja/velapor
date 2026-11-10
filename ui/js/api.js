// Talking to the Rust side. Every command and event the window uses is listed here.
//
// Opened in a plain browser (no Tauri), the window talks to a simulated machine instead,
// which is handy for working on the interface and for the website's screenshots.

let backend = window.__TAURI__ ?? null;

export class Api {
  /** The real Rust side, or the simulation outside the app. */
  static async backend() {
    if (!backend) backend = (await import("./demo/backend.js")).DemoBackend.create();
    return backend;
  }

  /** Whether the window runs on the simulated machine. */
  static get isDemo() {
    return !window.__TAURI__;
  }

  /** Calls a Rust command. */
  static async call(command, args) {
    const tauri = await this.backend();
    return tauri.core.invoke(command, args);
  }

  /** Runs `handler(payload)` whenever the Rust side sends `event`. */
  static async on(event, handler) {
    const tauri = await this.backend();
    return tauri.event.listen(event, ({ payload }) => handler(payload));
  }

  // ---- reading
  static getSnapshot() { return this.call("get_snapshot"); }
  static getHistory() { return this.call("get_history"); }
  static getDisplays() { return this.call("get_displays"); }
  static about() { return this.call("about"); }

  // ---- on request
  static runPreflight() { return this.call("run_preflight"); }
  static runSpeedTest() { return this.call("run_speed_test"); }

  // ---- links
  static openUrl(url) { return this.call("open_url", { url }); }
}
