//! The diagnostic engine: turns raw samples into findings a human can act on.
//!
//! Declarative rules (TOML) and code rules (Rust) will both implement [`Rule`].

use velapor_core::{Finding, Sample};

/// A check that looks at samples and may produce findings.
pub trait Rule {
    /// Stable identifier, e.g. "disk.smart.reallocated_sectors".
    fn id(&self) -> &'static str;
    /// Looks at the latest samples and returns anything worth telling the user.
    fn evaluate(&self, samples: &[Sample]) -> Vec<Finding>;
}

/// Holds every rule and runs them together.
#[derive(Default)]
pub struct Engine {
    rules: Vec<Box<dyn Rule>>,
}

impl Engine {
    /// Creates an engine with no rules.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a rule to the engine.
    pub fn register(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    /// Runs every rule and returns the findings, most serious first.
    pub fn evaluate(&self, samples: &[Sample]) -> Vec<Finding> {
        let mut findings: Vec<Finding> =
            self.rules.iter().flat_map(|rule| rule.evaluate(samples)).collect();
        findings.sort_by_key(|finding| std::cmp::Reverse(finding.severity));
        findings
    }
}
