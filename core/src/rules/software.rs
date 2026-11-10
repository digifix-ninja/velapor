//! Rules for the Software tab: the operating system and what starts with it.

use super::{Context, Outcome, Rule};
use crate::model::units::duration;
use crate::{Domain, Finding, Severity};

/// The software rules.
pub fn rules() -> Vec<Box<dyn Rule>> {
    vec![Box::new(LongUptime)]
}

/// Notices a machine that has not restarted for a long time.
pub struct LongUptime;

impl LongUptime {
    /// After how many days a restart is worth suggesting.
    const DAYS: u64 = 30;
}

impl Rule for LongUptime {
    /// Stable identifier.
    fn id(&self) -> &'static str {
        "software.uptime"
    }

    /// Belongs to Software.
    fn domain(&self) -> Domain {
        Domain::Software
    }

    /// Name in the list of checks.
    fn name(&self) -> &'static str {
        "Time since restart"
    }

    /// A notice after thirty days without a restart.
    fn check(&self, context: &Context) -> Outcome {
        let uptime = context.readings.software.uptime;
        if uptime == 0 {
            return Outcome::NotApplicable;
        }
        Outcome::from_option((uptime > Self::DAYS * 86_400).then(|| {
            Finding::new(self.id(), self.domain(), Severity::Notice, format!("Running for {} without a restart", duration(uptime)))
                .explain("Many system updates only take effect after a restart, and long-running systems slowly collect leftovers.")
                .recommend("Restart when it suits you.")
                .evidence("Up for", duration(uptime))
        }))
    }
}
