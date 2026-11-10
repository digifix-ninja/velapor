//! The menu-bar icon and its menu: the verdict at a glance, and the way back to the window.

use tauri::image::Image;
use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, Wry};
use velapor_core::watch::snapshot::Headline;

/// macOS: black-on-transparent template, tinted by the system for light and dark menu bars.
#[cfg(target_os = "macos")]
const TRAY_ICON: &[u8] = include_bytes!("../icons/tray.png");
/// Windows and Linux: a light mark with the amber flame, made for the usual dark taskbars.
#[cfg(not(target_os = "macos"))]
const TRAY_ICON: &[u8] = include_bytes!("../icons/tray-light.png");

/// The tray icon's identifier.
const TRAY_ID: &str = "velapor";

/// The menu item whose text follows the verdict.
pub struct Tray {
    status: MenuItem<Wry>,
}

impl Tray {
    /// Puts the icon in the menu bar (macOS) or system tray (Windows, Linux).
    pub fn install(app: &AppHandle) -> tauri::Result<()> {
        let status = MenuItem::with_id(app, "status", "Looking…", false, None::<&str>)?;
        let menu = Menu::with_items(
            app,
            &[
                &status,
                &PredefinedMenuItem::separator(app)?,
                &MenuItem::with_id(app, "open", "Open Velapor", true, None::<&str>)?,
                &MenuItem::with_id(app, "preflight", "Run Preflight", true, None::<&str>)?,
                &PredefinedMenuItem::separator(app)?,
                &MenuItem::with_id(app, "quit", "Quit Velapor", true, None::<&str>)?,
            ],
        )?;
        TrayIconBuilder::with_id(TRAY_ID)
            .icon(Image::from_bytes(TRAY_ICON)?)
            .icon_as_template(cfg!(target_os = "macos"))
            .tooltip("Velapor")
            .menu(&menu)
            .show_menu_on_left_click(true)
            .on_menu_event(Self::on_menu)
            .build(app)?;
        app.manage(Tray { status });
        Ok(())
    }

    /// Updates the status line and the tooltip with the latest verdict.
    pub fn refresh(app: &AppHandle, headline: &Headline) {
        let Some(tray) = app.try_state::<Tray>() else { return };
        let text = format!("{} · {}", headline.severity.label(), headline.title);
        let _ = tray.status.set_text(&text);
        if let Some(icon) = app.tray_by_id(TRAY_ID) {
            let _ = icon.set_tooltip(Some(format!("Velapor · {text}")));
        }
    }

    /// Handles a click on one of the menu items.
    fn on_menu(app: &AppHandle, event: MenuEvent) {
        match event.id.as_ref() {
            "open" => crate::show_main_window(app),
            "preflight" => {
                crate::show_main_window(app);
                let _ = app.emit("preflight", ());
            }
            "quit" => app.exit(0),
            _ => {}
        }
    }
}
