//! Rules: checks that turn readings into findings.
//!
//! Every rule implements [`Rule`]. A rule looks at the [`Context`] (the latest readings, the
//! last hour of history and the state at start) and answers with an [`Outcome`]. Rules never
//! read the machine themselves and never change anything; they only judge what was read.
//!
//! The rules live in one file per tab: [`hardware`], [`software`], [`network`],
//! [`performance`] and [`integrity`]. [`RuleBook`] holds them all.

pub mod hardware;
pub mod integrity;
pub mod network;
pub mod performance;
pub mod software;

use crate::clock::Millis;
use crate::watch::baseline::Baseline;
use crate::watch::history::History;
use crate::watch::readings::Readings;
use crate::{Domain, Finding, Severity};
use serde::Serialize;

/// Everything a rule may look at.
pub struct Context<'a> {
    /// The current time.
    pub now: Millis,
    /// The latest readings.
    pub readings: &'a Readings,
    /// The last hour.
    pub history: &'a History,
    /// What the machine looked like when Velapor started.
    pub baseline: &'a Baseline,
}

/// What a rule concluded.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// Looked, and everything is fine.
    Clear,
    /// Looked, and found something.
    Found(Vec<Finding>),
    /// Could not look: the reading does not exist on this machine (no battery, no sensors…).
    NotApplicable,
}

impl Outcome {
    /// `Found` when there are findings, `Clear` when there are none.
    pub fn from_findings(findings: Vec<Finding>) -> Self {
        if findings.is_empty() {
            Outcome::Clear
        } else {
            Outcome::Found(findings)
        }
    }

    /// The single finding if there is one, otherwise `Clear`.
    pub fn from_option(finding: Option<Finding>) -> Self {
        Self::from_findings(finding.into_iter().collect())
    }
}

/// One check Velapor runs on every tick.
pub trait Rule: Send {
    /// A stable identifier, e.g. `hardware.disk-space`.
    fn id(&self) -> &'static str;

    /// The tab the rule belongs to.
    fn domain(&self) -> Domain;

    /// A short name the window shows in the list of checks.
    fn name(&self) -> &'static str;

    /// Judges the readings.
    fn check(&self, context: &Context) -> Outcome;
}

/// How one rule came out, for the list of checks in the window and in Preflight.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleStatus {
    /// The rule's identifier.
    pub id: String,
    /// The tab it belongs to.
    pub domain: Domain,
    /// Its short name.
    pub name: String,
    /// The worst severity it found, `Clear` when it found nothing.
    pub severity: Severity,
    /// Whether the rule could look at all on this machine.
    pub applicable: bool,
}

/// The result of running every rule once.
#[derive(Debug, Clone, Default)]
pub struct Evaluation {
    /// Every finding, worst first.
    pub findings: Vec<Finding>,
    /// How each rule came out.
    pub statuses: Vec<RuleStatus>,
}

/// All rules Velapor knows.
pub struct RuleBook {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleBook {
    /// The full set of rules, tab by tab.
    pub fn standard() -> Self {
        let mut rules: Vec<Box<dyn Rule>> = Vec::new();
        rules.extend(hardware::rules());
        rules.extend(software::rules());
        rules.extend(network::rules());
        rules.extend(performance::rules());
        rules.extend(integrity::rules());
        Self { rules }
    }

    /// How many rules there are.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Whether the book is empty.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Runs every rule once and collects what they found.
    pub fn evaluate(&self, context: &Context) -> Evaluation {
        let mut evaluation = Evaluation::default();
        for rule in &self.rules {
            let outcome = rule.check(context);
            let (severity, applicable, mut found) = match outcome {
                Outcome::Clear => (Severity::Clear, true, Vec::new()),
                Outcome::NotApplicable => (Severity::Clear, false, Vec::new()),
                Outcome::Found(found) => {
                    let worst = found.iter().map(|f| f.severity).max().unwrap_or(Severity::Clear);
                    (worst, true, found)
                }
            };
            evaluation.statuses.push(RuleStatus {
                id: rule.id().to_string(),
                domain: rule.domain(),
                name: rule.name().to_string(),
                severity,
                applicable,
            });
            evaluation.findings.append(&mut found);
        }
        evaluation.findings.sort_by_key(|f| std::cmp::Reverse(f.severity));
        evaluation
    }
}

/// Picks a severity by comparing a value against a warning and a critical threshold.
pub(crate) fn grade(value: f64, warning: f64, critical: f64) -> Severity {
    if value >= critical {
        Severity::Critical
    } else if value >= warning {
        Severity::Warning
    } else {
        Severity::Clear
    }
}

/// Minutes as milliseconds.
pub(crate) const fn minutes(count: u64) -> Millis {
    count * 60 * 1000
}
