//! The beat: every two seconds the watchman reads the machine, and the result goes to the
//! window, the tray and, for new problems, a notification.

use crate::notifier::Notifier;
use crate::state::AppState;
use crate::tray::Tray;
use std::thread;
use tauri::{AppHandle, Emitter, Manager};
use velapor_core::Watchman;

/// Runs the watchman on its own thread for as long as the app runs.
pub struct Beat;

impl Beat {
    /// Starts the beat in the background.
    pub fn start(app: AppHandle) {
        thread::Builder::new()
            .name("velapor-beat".into())
            .spawn(move || loop {
                Self::once(&app);
                thread::sleep(Watchman::BEAT);
            })
            .expect("the beat thread can start");
    }

    /// One beat: read, judge, and pass the result on.
    fn once(app: &AppHandle) {
        let state = app.state::<AppState>();
        let tick = match state.watchman.lock() {
            Ok(mut watchman) => watchman.tick(),
            Err(_) => return,
        };
        Tray::refresh(app, &tick.snapshot.headline);
        Notifier::announce(app, &tick.alerts);
        let _ = app.emit("snapshot", &tick.snapshot);
        if let Ok(mut latest) = state.latest.lock() {
            *latest = Some(tick.snapshot);
        };
    }
}
