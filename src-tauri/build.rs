//! Lets Tauri prepare the app bundle (icons, permissions) before compiling.

/// Runs Tauri's build step.
fn main() {
    tauri_build::build()
}
