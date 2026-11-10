//! The battery, on laptops.

use super::Probe;
use serde::Serialize;
use starship_battery::units::{ratio::percent, thermodynamic_temperature::degree_celsius};
use starship_battery::{Manager, State};
use std::time::Duration;

/// The battery's condition and charge.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryReading {
    /// Charge, 0–100.
    pub charge: f32,
    /// Capacity left compared to when it was new, 0–100.
    pub health: f32,
    /// Charge cycles, when the system says.
    pub cycles: Option<u32>,
    /// "Charging", "Discharging", "Full", "Paused" (charge limit reached) or "Unknown".
    pub state: String,
    /// Temperature in °C, when the system says.
    pub temperature: Option<f32>,
    /// Maker and model, e.g. "SMP bq20z451".
    pub model: Option<String>,
    /// Battery chemistry, e.g. "lithium-ion".
    pub technology: String,
}

/// Reads the first battery, if the machine has one.
pub struct BatteryProbe {
    manager: Option<Manager>,
}

impl BatteryProbe {
    /// Connects to the system's battery information.
    pub fn new() -> Self {
        Self { manager: Manager::new().ok() }
    }

    /// Words for the battery's state.
    fn describe(state: State) -> &'static str {
        match state {
            State::Charging => "Charging",
            State::Discharging => "On battery",
            State::Full => "Full",
            State::Paused => "Charge limit reached",
            State::Empty => "Empty",
            _ => "Unknown",
        }
    }
}

impl Default for BatteryProbe {
    /// Same as [`BatteryProbe::new`].
    fn default() -> Self {
        Self::new()
    }
}

impl Probe for BatteryProbe {
    type Report = Option<BatteryReading>;

    /// Charge changes slowly.
    fn interval(&self) -> Duration {
        Duration::from_secs(20)
    }

    /// Reads the battery, or `None` on a machine without one.
    fn read(&mut self) -> Option<BatteryReading> {
        let manager = self.manager.as_ref()?;
        let battery = manager.batteries().ok()?.flatten().next()?;
        let model = [battery.vendor(), battery.model()].into_iter().flatten().collect::<Vec<_>>().join(" ");
        Some(BatteryReading {
            charge: battery.state_of_charge().get::<percent>(),
            health: battery.state_of_health().get::<percent>(),
            cycles: battery.cycle_count(),
            state: Self::describe(battery.state()).to_string(),
            temperature: battery.temperature().map(|t| t.get::<degree_celsius>()),
            model: (!model.trim().is_empty()).then(|| model.trim().to_string()),
            technology: battery.technology().to_string(),
        })
    }
}
