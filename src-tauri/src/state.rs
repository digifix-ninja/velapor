//! What every command can reach.

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use velapor_core::{Snapshot, Watchman};

/// Shared between the beat, the commands and the tray.
pub struct AppState {
    /// The watchman. Locked briefly on every beat and for Preflight.
    pub watchman: Arc<Mutex<Watchman>>,
    /// The latest snapshot, so the window gets an answer at once when it opens.
    pub latest: Arc<Mutex<Option<Snapshot>>>,
    /// Whether a speed test is running, so two never run at once.
    pub speed_test_running: Arc<AtomicBool>,
}

impl AppState {
    /// Creates the watchman. Nothing is read until the first beat.
    pub fn new() -> Self {
        Self {
            watchman: Arc::new(Mutex::new(Watchman::new())),
            latest: Arc::new(Mutex::new(None)),
            speed_test_running: Arc::new(AtomicBool::new(false)),
        }
    }
}
