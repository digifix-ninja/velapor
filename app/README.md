# Velapor app

The Tauri application lives here: a small Rust backend that talks to Sentry over IPC, and a light web
frontend for the cockpit, Preflight and alerts. It runs as the normal user and never touches the system
directly.

It will be scaffolded in phase 0. The first visual concept is in
[`docs/design/cockpit.html`](../docs/design/cockpit.html).
