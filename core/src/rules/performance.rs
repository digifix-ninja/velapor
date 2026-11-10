//! Rules for the Performance tab: lasting load, memory pressure, swap and hungry processes.

use super::{minutes, Context, Outcome, Rule};
use crate::model::units::{bytes, percent};
use crate::{Domain, Finding, Severity};

/// The performance rules.
pub fn rules() -> Vec<Box<dyn Rule>> {
    vec![Box::new(SustainedLoad), Box::new(MemoryPressure), Box::new(Swap), Box::new(HungryProcess)]
}

/// Warns when the processor stays busy for minutes, not just for a moment.
pub struct SustainedLoad;

impl SustainedLoad {
    /// How long the load has to last before it counts.
    const WINDOW: u64 = minutes(5);
}

impl Rule for SustainedLoad {
    /// Stable identifier.
    fn id(&self) -> &'static str {
        "performance.sustained-load"
    }

    /// Belongs to Performance.
    fn domain(&self) -> Domain {
        Domain::Performance
    }

    /// Name in the list of checks.
    fn name(&self) -> &'static str {
        "Lasting processor load"
    }

    /// Notice when the five-minute average is above 85 %, warning above 95 %. Waits until
    /// Velapor has watched for five minutes.
    fn check(&self, context: &Context) -> Outcome {
        if context.history.span(context.now) < Self::WINDOW - minutes(1) / 2 {
            return Outcome::Clear;
        }
        let Some(average) = context.history.average(context.now, Self::WINDOW, |s| s.cpu) else {
            return Outcome::Clear;
        };
        let severity = match average {
            a if a >= 95.0 => Severity::Warning,
            a if a >= 85.0 => Severity::Notice,
            _ => Severity::Clear,
        };
        let busiest = context.readings.performance.by_cpu.first();
        Outcome::from_option((severity > Severity::Clear).then(|| {
            let mut finding = Finding::new(self.id(), self.domain(), severity, format!("Processor has been {} busy for five minutes", percent(average as f64)))
                .explain("Everything else has to wait while the processor is this busy, so the machine feels slow and runs warm.")
                .recommend("Check the busiest process below. If you don't need it right now, close it from its own window.")
                .evidence("Five-minute average", percent(average as f64));
            if let Some(process) = busiest {
                finding = finding
                    .cause(format!("{} uses the most processor time right now.", process.name))
                    .evidence("Busiest process", format!("{} ({})", process.name, percent(process.cpu as f64)));
            }
            finding
        }))
    }
}

/// Warns when memory is nearly used up.
pub struct MemoryPressure;

impl Rule for MemoryPressure {
    /// Stable identifier.
    fn id(&self) -> &'static str {
        "performance.memory-pressure"
    }

    /// Belongs to Performance.
    fn domain(&self) -> Domain {
        Domain::Performance
    }

    /// Name in the list of checks.
    fn name(&self) -> &'static str {
        "Memory pressure"
    }

    /// Warning below 8 % available memory, critical below 3 %. Measures what is available,
    /// not what is used, because systems keep memory busy with caches on purpose.
    fn check(&self, context: &Context) -> Outcome {
        let memory = &context.readings.processor.memory;
        if memory.total == 0 {
            return Outcome::NotApplicable;
        }
        let available = memory.available as f64 / memory.total as f64 * 100.0;
        let severity = match available {
            a if a < 3.0 => Severity::Critical,
            a if a < 8.0 => Severity::Warning,
            _ => Severity::Clear,
        };
        let hungriest = context.readings.performance.by_memory.first();
        Outcome::from_option((severity > Severity::Clear).then(|| {
            let mut finding = Finding::new(self.id(), self.domain(), severity, format!("Only {} of memory is left", bytes(memory.available)))
                .explain("When memory runs out the system moves work to the drive, which is far slower, and programs may be closed.")
                .recommend("Close programs or browser tabs you are not using.")
                .evidence("Available", format!("{} of {}", bytes(memory.available), bytes(memory.total)));
            if let Some(process) = hungriest {
                finding = finding
                    .cause(format!("{} holds the most memory.", process.name))
                    .evidence("Largest process", format!("{} ({})", process.name, bytes(process.memory)));
            }
            finding
        }))
    }
}

/// Notices heavy use of swap.
pub struct Swap;

impl Rule for Swap {
    /// Stable identifier.
    fn id(&self) -> &'static str {
        "performance.swap"
    }

    /// Belongs to Performance.
    fn domain(&self) -> Domain {
        Domain::Performance
    }

    /// Name in the list of checks.
    fn name(&self) -> &'static str {
        "Swap use"
    }

    /// Notice above 50 % of swap in use, warning above 85 %.
    fn check(&self, context: &Context) -> Outcome {
        let memory = &context.readings.processor.memory;
        if memory.swap_total == 0 {
            return Outcome::NotApplicable;
        }
        let used = memory.swap_percent();
        let severity = match used {
            u if u >= 85.0 => Severity::Warning,
            u if u >= 50.0 => Severity::Notice,
            _ => Severity::Clear,
        };
        Outcome::from_option((severity > Severity::Clear).then(|| {
            Finding::new(self.id(), self.domain(), severity, format!("{} of swap is in use", percent(used)))
                .explain(
                    "Swap is drive space used as overflow memory. Heavy use means memory has been short for a while.",
                )
                .cause("More programs are open than the memory comfortably holds.")
                .recommend("Close what you don't need. If this is normal for your work, more memory would help.")
                .evidence("Swap", format!("{} of {}", bytes(memory.swap_used), bytes(memory.swap_total)))
        }))
    }
}

/// Notices a single process holding a large share of memory.
pub struct HungryProcess;

impl Rule for HungryProcess {
    /// Stable identifier.
    fn id(&self) -> &'static str {
        "performance.hungry-process"
    }

    /// Belongs to Performance.
    fn domain(&self) -> Domain {
        Domain::Performance
    }

    /// Name in the list of checks.
    fn name(&self) -> &'static str {
        "Processes using a lot of memory"
    }

    /// A notice for any process holding more than 30 % of all memory.
    fn check(&self, context: &Context) -> Outcome {
        let total = context.readings.processor.memory.total;
        if total == 0 {
            return Outcome::NotApplicable;
        }
        let findings = context
            .readings
            .performance
            .by_memory
            .iter()
            .filter(|p| p.memory as f64 / total as f64 > 0.30)
            .map(|process| {
                let share = process.memory as f64 / total as f64 * 100.0;
                Finding::new(format!("{}:{}", self.id(), process.name), self.domain(), Severity::Notice, format!("{} uses {} of memory", process.name, percent(share)))
                    .explain("One program is holding a large part of the memory. That is fine for heavy work, but it can also be a leak.")
                    .recommend("If the program has been open for a long time, restarting it usually gives the memory back.")
                    .evidence("Process", format!("{} (pid {})", process.name, process.pid))
                    .evidence("Memory", bytes(process.memory))
            })
            .collect();
        Outcome::from_findings(findings)
    }
}
