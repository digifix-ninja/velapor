//! Everything the window can ask for. None of it changes the machine. Errors reach the window
//! as plain sentences.
//!
//! Commands that take a few seconds (Preflight, the speed test) are `async` and do their work
//! on a background thread, so the window never freezes.

use crate::about::About;
use crate::desktop::Desktop;
use crate::displays::Displays;
use crate::state::AppState;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, State};
use velapor_core::probes::AttachedDevice;
use velapor_core::watch::history::Sample;
use velapor_core::watch::preflight::ConnectionChecks;
use velapor_core::{clock, PreflightReport, Snapshot, SpeedResult, SpeedTest};

/// What a command answers: a value, or a sentence explaining what went wrong.
type Answer<T> = Result<T, String>;

/// The latest snapshot, or nothing if the first beat has not finished yet.
#[tauri::command]
pub fn get_snapshot(state: State<AppState>) -> Option<Snapshot> {
    state.latest.lock().ok().and_then(|latest| latest.clone())
}

/// The last hour of samples, for the trend charts.
#[tauri::command]
pub fn get_history(state: State<AppState>) -> Vec<Sample> {
    state.watchman.lock().map(|w| w.history()).unwrap_or_default()
}

/// The connected screens.
#[tauri::command]
pub fn get_displays(app: AppHandle) -> Vec<AttachedDevice> {
    Displays::read(&app)
}

/// A full check: fresh readings, every rule, and the connection to the internet.
#[tauri::command]
pub async fn run_preflight(state: State<'_, AppState>) -> Answer<PreflightReport> {
    let watchman = state.watchman.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let started = clock::now();
        // The connection checks run without holding the watchman, so the beat goes on.
        let connection = ConnectionChecks::run();
        let evaluation = watchman.lock().map_err(|_| "The watchman is not available.".to_string())?.evaluate_fresh();
        Ok(PreflightReport::new(started, evaluation, connection))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Measures the internet connection, sending "speed-progress" events while it runs.
#[tauri::command]
pub async fn run_speed_test(app: AppHandle, state: State<'_, AppState>) -> Answer<SpeedResult> {
    let running = state.speed_test_running.clone();
    if running.swap(true, Ordering::SeqCst) {
        return Err("A speed test is already running.".into());
    }
    let result = tauri::async_runtime::spawn_blocking(move || {
        SpeedTest::new().run(|progress| {
            let _ = app.emit("speed-progress", progress);
        })
    })
    .await
    .map_err(|e| e.to_string());
    running.store(false, Ordering::SeqCst);
    result?
}

/// Name, version, developer and links for the About panel.
#[tauri::command]
pub fn about(app: AppHandle) -> About {
    About::of(&app)
}

/// Opens one of Velapor's own links in the browser. Other addresses are refused.
#[tauri::command]
pub fn open_url(url: String) -> Answer<()> {
    if !About::may_open(&url) {
        return Err("Velapor only opens its own links.".into());
    }
    Desktop::open_url(&url)
}
