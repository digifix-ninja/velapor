//! Temperature sensors: processor, graphics, drives, battery.

use super::Probe;
use serde::Serialize;
use std::time::Duration;
use sysinfo::Components;

/// One temperature sensor.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorReading {
    /// The sensor's name as the system gives it.
    pub label: String,
    /// The temperature now, in °C.
    pub celsius: f32,
    /// The highest temperature seen since Velapor started.
    pub highest: Option<f32>,
    /// The temperature the hardware itself calls critical, when it says.
    pub critical: Option<f32>,
}

/// Reads the temperature sensors the system exposes.
pub struct SensorProbe {
    components: Option<Components>,
}

impl SensorProbe {
    /// Finds the sensors.
    pub fn new() -> Self {
        Self { components: Components::new_with_refreshed_list().ok() }
    }
}

impl Default for SensorProbe {
    /// Same as [`SensorProbe::new`].
    fn default() -> Self {
        Self::new()
    }
}

impl Probe for SensorProbe {
    type Report = Vec<SensorReading>;

    /// Temperatures move within seconds.
    fn interval(&self) -> Duration {
        Duration::from_secs(2)
    }

    /// Reads each sensor that reports a believable temperature, hottest first.
    fn read(&mut self) -> Vec<SensorReading> {
        let Some(components) = self.components.as_mut() else { return Vec::new() };
        components.refresh(true);
        let mut readings: Vec<SensorReading> = components
            .iter()
            .filter_map(|c| {
                let celsius = c.temperature()?;
                (celsius > 5.0 && celsius < 150.0).then(|| SensorReading {
                    label: c.label().to_string(),
                    celsius,
                    highest: c.max(),
                    critical: c.critical().filter(|t| *t > 40.0 && *t < 150.0),
                })
            })
            .collect();
        readings.sort_by(|a, b| b.celsius.total_cmp(&a.celsius));
        readings
    }
}
