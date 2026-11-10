//! The five areas Velapor watches.

use serde::Serialize;

/// The area of the machine a reading or finding belongs to. Each one is a tab in the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Domain {
    /// Processor, memory, drives, sensors, battery and attached devices.
    Hardware,
    /// The operating system and what starts with it.
    Software,
    /// Interfaces, open ports, connections and traffic.
    Network,
    /// Load, memory pressure and hungry processes.
    Performance,
    /// Security settings and the parts of the system that should stay still.
    Integrity,
}

impl Domain {
    /// Every domain, in the order the window shows them.
    pub const ALL: [Domain; 5] =
        [Domain::Hardware, Domain::Software, Domain::Network, Domain::Performance, Domain::Integrity];

    /// The name shown on the tab.
    pub fn label(self) -> &'static str {
        match self {
            Domain::Hardware => "Hardware",
            Domain::Software => "Software",
            Domain::Network => "Network",
            Domain::Performance => "Performance",
            Domain::Integrity => "Integrity",
        }
    }
}
