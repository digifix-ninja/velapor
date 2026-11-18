//! Facts shown in the About panel. Change them here (and in LICENSE) when needed.

use serde::Serialize;
use tauri::AppHandle;

/// The app's own website.
pub const APP_WEBSITE: &str = "https://velapor.app";
/// Who makes Velapor.
pub const DEVELOPER: &str = "DigiFix S.à r.l.-S.";
/// The developer's website.
pub const WEBSITE: &str = "https://digifix.ninja";
/// The family of apps Velapor belongs to (DigiFix makes them all, so it is the same site).
pub const FAMILY: &str = "https://digifix.ninja";
/// Public source code on GitHub (an empty value hides the link).
pub const SOURCE_CODE: &str = "https://github.com/digifix-ninja/velapor";
/// The licence.
pub const LICENCE: &str = "MIT";
/// The year in the copyright line.
pub const COPYRIGHT_YEAR: &str = "2026";

/// Everything the About panel shows.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct About {
    name: String,
    version: String,
    app_website: &'static str,
    developer: &'static str,
    website: &'static str,
    family: &'static str,
    source_code: Option<&'static str>,
    licence: &'static str,
    copyright_year: &'static str,
}

impl About {
    /// Name and version come from the app bundle, the rest from the constants above.
    pub fn of(app: &AppHandle) -> Self {
        let info = app.package_info();
        Self {
            name: info.name.clone(),
            version: info.version.to_string(),
            app_website: APP_WEBSITE,
            developer: DEVELOPER,
            website: WEBSITE,
            family: FAMILY,
            source_code: (!SOURCE_CODE.is_empty()).then_some(SOURCE_CODE),
            licence: LICENCE,
            copyright_year: COPYRIGHT_YEAR,
        }
    }

    /// Websites the app may open in the browser: its own, the developer's, the family's and
    /// the source code.
    pub fn may_open(url: &str) -> bool {
        [APP_WEBSITE, WEBSITE, FAMILY, SOURCE_CODE]
            .iter()
            .filter(|site| !site.is_empty())
            .any(|site| url == *site || url.starts_with(&format!("{site}/")))
    }
}
