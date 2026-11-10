//! System notifications: only for new warnings and critical findings. Notices wait quietly
//! in the window.

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;
use velapor_core::Finding;

/// Tells the person about new problems.
pub struct Notifier;

impl Notifier {
    /// Shows one notification per new finding (at most three at once; the rest are in the
    /// window).
    pub fn announce(app: &AppHandle, findings: &[Finding]) {
        for finding in findings.iter().filter(|f| f.severity.deserves_notification()).take(3) {
            let result = app
                .notification()
                .builder()
                .title(format!("{} · {}", finding.severity.label(), finding.title))
                .body(&finding.recommendation)
                .show();
            if let Err(error) = result {
                log::warn!("couldn't show a notification: {error}");
            }
        }
    }
}
