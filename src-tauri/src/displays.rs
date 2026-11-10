//! Connected screens. The window system knows them best, so they are read here rather than
//! in the core, and shown on the Hardware tab next to the other attached devices.

use tauri::{AppHandle, Manager};
use velapor_core::probes::AttachedDevice;

/// Reads the connected screens.
pub struct Displays;

impl Displays {
    /// Every screen the window system reports, with its resolution and scale.
    pub fn read(app: &AppHandle) -> Vec<AttachedDevice> {
        let Some(window) = app.get_webview_window("main") else {
            return Vec::new();
        };
        let primary = window.primary_monitor().ok().flatten().map(|m| *m.position());
        let monitors = window.available_monitors().unwrap_or_default();
        monitors
            .iter()
            .enumerate()
            .map(|(index, monitor)| {
                let size = monitor.size();
                let name = monitor.name().cloned().unwrap_or_else(|| format!("Display {}", index + 1));
                let is_primary = primary.as_ref() == Some(monitor.position());
                AttachedDevice {
                    key: format!("display:{name}:{}x{}", size.width, size.height),
                    name: name.clone(),
                    maker: None,
                    kind: if is_primary { "Main display".into() } else { "Display".into() },
                    connection: "Display".into(),
                    detail: format!("{} × {} · scale {}", size.width, size.height, monitor.scale_factor()),
                }
            })
            .collect()
    }
}
