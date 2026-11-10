# Developing Velapor

Velapor is a [Tauri 2](https://tauri.app) desktop app: a Rust core that reads and judges the
machine (`core/`), a thin Rust app around it (`src-tauri/`) and a plain HTML/CSS/JavaScript
interface (`ui/`). There's no Node.js and no frontend build step. How the parts work together is
described in [ARCHITECTURE.md](ARCHITECTURE.md).

## Requirements

- **Rust 1.85+** (`rustup update`)
- **Tauri CLI:** `cargo install tauri-cli --version "^2" --locked`
- Per system:
  - **macOS:** Xcode command line tools (`xcode-select --install`)
  - **Windows:** Microsoft C++ Build Tools ("Desktop development with C++"); WebView2 is built
    into Windows 10/11
  - **Linux (Debian/Ubuntu):**
    `sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf libdbus-1-dev pkg-config build-essential`

## Running, testing and building

```sh
cargo tauri dev                           # run in development
cargo test -p velapor-core                # the core's tests, including the read-only guard
cargo run -p velapor-core --example snapshot   # one snapshot of this machine, as JSON
cargo tauri build                         # installers for the system you're on
```

**Working on the interface only?** Serve `ui/` with any local web server
(`python3 -m http.server -d ui`) and open it in a browser. Outside the app, the window talks to a
simulated machine (`ui/js/demo/backend.js`) with live gauges, a nearly full drive, and a new
program that starts listening on the network a while after opening.

Each system builds its own installers; a Mac can't build the Windows or Linux versions locally.

| | macOS | Windows | Linux |
|---|---|---|---|
| Installers | `.app`, `.dmg` | `.msi`, `-setup.exe` | `.deb`, `.rpm`, `.AppImage` |

If the project sits on a disk without file-lock support (some external or network drives),
incremental compilation is already switched off in `Cargo.toml`. For faster builds, put the
build output on the internal disk, e.g. `export CARGO_TARGET_DIR="$HOME/Library/Caches/velapor-target"`.

## Releases (all three systems)

`.github/workflows/build.yml` runs the core's tests and builds macOS (Apple Silicon and Intel),
Windows and Linux on GitHub's servers.

- **Push a version tag** (`git tag v0.1.0 && git push origin v0.1.0`): the installers are attached
  to a **draft** release. Nothing is public until you press *Publish*. They always have the same
  names (`Velapor-macOS-Apple-Silicon.dmg`, `Velapor-Windows-Setup.exe`, `Velapor-Linux.deb`…), so
  `releases/latest/download/<name>` always gives the newest version.
- **Run it by hand** under *Actions → Build Velapor → Run workflow*: the installers are kept as
  downloadable artifacts of the run.

### Signing the macOS version

The workflow signs and notarises the macOS installers when the repository has these secrets
(*Settings → Secrets and variables → Actions*):

| Secret | What it is |
|---|---|
| `APPLE_CERTIFICATE` | the *Developer ID Application* certificate as a `.p12`, base64-encoded |
| `APPLE_CERTIFICATE_PASSWORD` | the password chosen when exporting the `.p12` |
| `APPLE_SIGNING_IDENTITY` | e.g. `Developer ID Application: Name (TEAMID)` |
| `APPLE_ID` | the Apple ID e-mail of the developer account |
| `APPLE_PASSWORD` | an app-specific password for that Apple ID (account.apple.com) |
| `APPLE_TEAM_ID` | the 10-character team ID |

Without them (in a fork, for example), the app gets an ad-hoc signature: it works, but macOS asks
to confirm it under *System Settings → Privacy & Security → Open Anyway*.

## Project layout

```
core/src/                   reads and judges the machine (no UI, never writes)
  model/                      Finding, Evidence, Severity, Domain, units
  probes/                     one probe per part: machine, processor, storage, sensors, battery,
                              devices, network, processes, software, posture, fingerprints
  rules/                      Rule, RuleBook, and the rules per tab
  watch/                      Watchman, Readings, History, Baseline, Ledger, Snapshot, Preflight
  net/                        ReachabilityCheck, SpeedTest
  system_query.rs             SystemQuery: the fixed list of read-only system tools
core/tests/                 read-only guard and end-to-end watchman tests
core/examples/snapshot.rs   prints one snapshot as JSON
src-tauri/src/              the desktop app (macOS, Windows, Linux)
  main.rs                     start-up and wiring
  watch.rs                    Beat: the two-second tick → window, tray, notifications
  commands.rs                 everything the window can ask for
  state.rs                    AppState shared by commands and the beat
  tray.rs                     Tray: menu-bar / system-tray icon and menu
  notifier.rs                 Notifier: which findings become notifications
  displays.rs                 attached displays, read from the window system
  desktop.rs                  Desktop: opens the app's own links in the browser
  about.rs                    About: version, websites, developer, licence
src-tauri/tauri.conf.json   app configuration
ui/                         the interface (plain HTML/CSS/JS modules)
  index.html                  every tab's skeleton, filled by the views
  css/                        tokens (colours, light and dark), base, cockpit, overlays
  js/main.js                  App: creates the views, connects to Rust
  js/api.js                   Api: every command and event
  js/store.js                 Store: the latest snapshot, history and speed tests
  js/views/                   one view per tab, plus header, tabs, sheets and toast
  js/widgets/                 Gauge, Trend, SpeedDial
  js/demo/backend.js          simulated machine for working on the UI in a browser
.github/workflows/build.yml  tests and installers for all three systems
docs/                       these guides and the README screenshot
```

Code style: small objects with one job each, and a short comment on every function.

## Changing the About information

Websites, developer and the link to the source code are constants in
`src-tauri/src/about.rs`. The licence text is in `LICENSE`.
