//! Rules for the Hardware tab: drives, temperature, battery and attached devices.

use super::{grade, Context, Outcome, Rule};
use crate::model::units::{bytes, percent};
use crate::{Domain, Finding, Severity};

/// The hardware rules.
pub fn rules() -> Vec<Box<dyn Rule>> {
    vec![Box::new(DiskSpace), Box::new(Temperature), Box::new(BatteryHealth), Box::new(NewDevice)]
}

/// Warns when a drive is nearly full.
pub struct DiskSpace;

impl Rule for DiskSpace {
    /// Stable identifier.
    fn id(&self) -> &'static str {
        "hardware.disk-space"
    }

    /// Belongs to Hardware.
    fn domain(&self) -> Domain {
        Domain::Hardware
    }

    /// Name in the list of checks.
    fn name(&self) -> &'static str {
        "Free space on drives"
    }

    /// Warning above 90 % full, critical above 97 %. Read-only drives are skipped: they are
    /// meant to be full (system snapshots, installers).
    fn check(&self, context: &Context) -> Outcome {
        let disks = &context.readings.disks;
        if disks.is_empty() {
            return Outcome::NotApplicable;
        }
        let findings = disks
            .iter()
            .filter(|d| !d.read_only && d.total > 0)
            .filter_map(|disk| {
                let used = disk.used_percent();
                let severity = grade(used, 90.0, 97.0);
                (severity > Severity::Clear).then(|| {
                    Finding::new(format!("{}:{}", self.id(), disk.mount_point), self.domain(), severity, format!("{} is {} full", disk.name, percent(used)))
                        .explain("When a drive runs out of space, programs cannot save their work and the system slows down or stops updating.")
                        .cause("Large downloads, caches, old backups or virtual machines usually take the most room.")
                        .recommend("Look at what takes the most space on this drive and move or remove what you no longer need.")
                        .evidence("Drive", &disk.mount_point)
                        .evidence("Free", format!("{} of {}", bytes(disk.available), bytes(disk.total)))
                })
            })
            .collect();
        Outcome::from_findings(findings)
    }
}

/// Warns when a sensor runs hot.
pub struct Temperature;

impl Temperature {
    /// The temperature that counts as too hot for a sensor: its own critical value if it
    /// reports one, otherwise 95 °C.
    fn limit(critical: Option<f32>) -> f32 {
        critical.filter(|c| *c > 40.0).unwrap_or(95.0)
    }
}

impl Rule for Temperature {
    /// Stable identifier.
    fn id(&self) -> &'static str {
        "hardware.temperature"
    }

    /// Belongs to Hardware.
    fn domain(&self) -> Domain {
        Domain::Hardware
    }

    /// Name in the list of checks.
    fn name(&self) -> &'static str {
        "Temperature"
    }

    /// Warning within 10 °C of the limit, critical at the limit. Reports the hottest sensor.
    fn check(&self, context: &Context) -> Outcome {
        let Some(sensor) = context.readings.hottest_sensor() else {
            return Outcome::NotApplicable;
        };
        let limit = Self::limit(sensor.critical);
        let severity = grade(sensor.celsius as f64, (limit - 10.0) as f64, limit as f64);
        Outcome::from_option((severity > Severity::Clear).then(|| {
            Finding::new(self.id(), self.domain(), severity, format!("{} is running hot", sensor.label))
                .explain("Hot parts slow themselves down to protect themselves, and heat over a long time wears them out.")
                .cause("Heavy work, blocked air vents, dust, or a soft surface under the machine.")
                .recommend("Give the machine air, check which program keeps it busy, and clean the vents if this keeps happening.")
                .evidence("Sensor", &sensor.label)
                .evidence("Now", format!("{:.0} °C", sensor.celsius))
                .evidence("Limit", format!("{limit:.0} °C"))
        }))
    }
}

/// Notices a worn battery.
pub struct BatteryHealth;

impl Rule for BatteryHealth {
    /// Stable identifier.
    fn id(&self) -> &'static str {
        "hardware.battery-health"
    }

    /// Belongs to Hardware.
    fn domain(&self) -> Domain {
        Domain::Hardware
    }

    /// Name in the list of checks.
    fn name(&self) -> &'static str {
        "Battery health"
    }

    /// Notice below 80 % of the original capacity, warning below 60 %.
    fn check(&self, context: &Context) -> Outcome {
        let Some(battery) = &context.readings.battery else {
            return Outcome::NotApplicable;
        };
        let severity = match battery.health {
            h if h < 60.0 => Severity::Warning,
            h if h < 80.0 => Severity::Notice,
            _ => Severity::Clear,
        };
        let mut finding = (severity > Severity::Clear).then(|| {
            Finding::new(self.id(), self.domain(), severity, format!("Battery holds {} of its original charge", percent(battery.health as f64)))
                .explain("Batteries wear with every charge cycle. A worn battery runs out sooner and may shut the machine down without warning.")
                .cause("Age and the number of charge cycles.")
                .recommend("Plan for a battery replacement if the machine no longer lasts as long as you need.")
                .evidence("Health", percent(battery.health as f64))
        });
        if let (Some(f), Some(cycles)) = (finding.as_mut(), battery.cycles) {
            f.evidence.push(crate::Evidence::new("Charge cycles", cycles.to_string()));
        }
        Outcome::from_option(finding)
    }
}

/// Notices devices attached since Velapor started.
pub struct NewDevice;

impl Rule for NewDevice {
    /// Stable identifier.
    fn id(&self) -> &'static str {
        "hardware.new-device"
    }

    /// Belongs to Hardware.
    fn domain(&self) -> Domain {
        Domain::Hardware
    }

    /// Name in the list of checks.
    fn name(&self) -> &'static str {
        "Newly attached devices"
    }

    /// A notice for every device that was not attached when Velapor started.
    fn check(&self, context: &Context) -> Outcome {
        let findings = context
            .readings
            .devices
            .iter()
            .filter(|d| !context.baseline.knows_device(&d.key))
            .map(|device| {
                Finding::new(format!("{}:{}", self.id(), device.key), self.domain(), Severity::Notice, format!("{} was attached", device.name))
                    .explain("A device was connected after Velapor started. This is usually you, and then there is nothing to do.")
                    .recommend("If you did not plug this in, find out where it came from before you trust it.")
                    .evidence("Device", &device.name)
                    .evidence("Connection", &device.connection)
                    .evidence("Kind", &device.kind)
            })
            .collect();
        Outcome::from_findings(findings)
    }
}
