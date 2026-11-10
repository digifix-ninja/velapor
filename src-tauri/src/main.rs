//! Velapor for macOS, Windows and Linux: the window, the tray icon and the watch around
//! `velapor-core`.
//!
//! * `main.rs`     – starts the app and wires everything together
//! * `state.rs`    – what every command can reach (`AppState`)
//! * `watch.rs`    – the background beat that reads the machine every two seconds
//! * `commands.rs` – everything the window can ask for
//! * `displays.rs` – connected screens, which only the window system knows
//! * `tray.rs`     – the menu-bar / system-tray icon and its menu
//! * `notifier.rs` – system notifications for new warnings and critical findings
//! * `desktop.rs`  – opening links in the browser, per operating system
//! * `about.rs`    – name, version, developer and licence for the About panel
//!
//! Velapor only reads. It keeps everything in memory and stores nothing.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod about;
mod commands;
mod desktop;
mod displays;
mod notifier;
mod state;
mod tray;
mod watch;

use state::AppState;
use tauri::{AppHandle, Manager, RunEvent, WindowEvent};
use tray::Tray;
use watch::Beat;

/// Starts Velapor.
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let handle = app.handle().clone();
            app.manage(AppState::new());
            Tray::install(&handle)?;
            Beat::start(handle);
            Ok(())
        })
        .on_window_event(|window, event| {
            // A watchman keeps watching: closing the window leaves Velapor in the tray.
            // Linux desktops don't always show a tray, so there closing really quits.
            if let WindowEvent::CloseRequested { api, .. } = event {
                if cfg!(not(target_os = "linux")) {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::get_history,
            commands::get_displays,
            commands::run_preflight,
            commands::run_speed_test,
            commands::about,
            commands::open_url,
        ])
        .build(tauri::generate_context!())
        .expect("error while starting Velapor")
        .run(on_run_event);
}

/// Clicking the Dock icon brings a hidden window back.
fn on_run_event(app: &AppHandle, event: RunEvent) {
    #[cfg(target_os = "macos")]
    if let RunEvent::Reopen { .. } = event {
        show_main_window(app);
    }
    let _ = (app, &event);
}

/// Shows, un-minimises and focuses the main window.
pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
