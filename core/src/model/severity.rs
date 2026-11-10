//! How serious something is, from calm to urgent.

use serde::Serialize;

/// How serious a finding is. Ordered from calm to urgent, so the worst one is the maximum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Everything looks fine.
    Clear,
    /// Worth knowing, nothing to do yet.
    Notice,
    /// Keep an eye on it.
    Warning,
    /// Look at this now.
    Critical,
}

impl Severity {
    /// The word the window shows next to the colour and icon.
    pub fn label(self) -> &'static str {
        match self {
            Severity::Clear => "Clear",
            Severity::Notice => "Notice",
            Severity::Warning => "Warning",
            Severity::Critical => "Critical",
        }
    }

    /// Whether this is serious enough for a system notification.
    pub fn deserves_notification(self) -> bool {
        self >= Severity::Warning
    }
}
