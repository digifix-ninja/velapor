//! What the window shows: the state of the machine at one moment, already judged.

use super::readings::Readings;
use crate::clock::Millis;
use crate::rules::{Evaluation, RuleStatus};
use crate::{Domain, Finding, Severity};
use serde::Serialize;

/// The one-line verdict at the top of the window.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Headline {
    /// The worst severity of all findings.
    pub severity: Severity,
    /// The headline itself.
    pub title: String,
    /// One sentence under it.
    pub detail: String,
}

impl Headline {
    /// Writes the headline for a set of findings (worst first).
    pub fn for_findings(findings: &[Finding]) -> Self {
        let Some(worst) = findings.first() else {
            return Self {
                severity: Severity::Clear,
                title: "All quiet".to_string(),
                detail: "Everything Velapor watches looks fine.".to_string(),
            };
        };
        let others = findings.len() - 1;
        let detail = match others {
            0 => "This is the only thing Velapor would look at.".to_string(),
            1 => "There is one more thing below.".to_string(),
            n => format!("There are {n} more things below."),
        };
        Self { severity: worst.severity, title: worst.title.clone(), detail }
    }
}

/// How one tab is doing, for the tab bar and the overview.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainSummary {
    /// Which tab.
    pub domain: Domain,
    /// Its name.
    pub label: String,
    /// The worst severity in it.
    pub severity: Severity,
    /// How many findings it has.
    pub findings: usize,
    /// How many checks ran in it.
    pub checks: usize,
    /// How many of those found nothing.
    pub clear: usize,
}

impl DomainSummary {
    /// Summarises one tab from the evaluation.
    fn of(domain: Domain, evaluation: &Evaluation) -> Self {
        let statuses: Vec<&RuleStatus> =
            evaluation.statuses.iter().filter(|s| s.domain == domain && s.applicable).collect();
        let findings: Vec<&Finding> = evaluation.findings.iter().filter(|f| f.domain == domain).collect();
        Self {
            domain,
            label: domain.label().to_string(),
            severity: findings.iter().map(|f| f.severity).max().unwrap_or(Severity::Clear),
            findings: findings.len(),
            checks: statuses.len(),
            clear: statuses.iter().filter(|s| s.severity == Severity::Clear).count(),
        }
    }
}

/// Everything the window needs to draw itself once.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    /// When it was taken.
    pub at: Millis,
    /// Since when Velapor has been watching.
    pub watching_since: Millis,
    /// The verdict.
    pub headline: Headline,
    /// One summary per tab.
    pub domains: Vec<DomainSummary>,
    /// Every finding, worst first.
    pub findings: Vec<Finding>,
    /// How each check came out.
    pub checks: Vec<RuleStatus>,
    /// The readings behind it all.
    pub readings: Readings,
}

impl Snapshot {
    /// Puts a snapshot together from an evaluation and the readings it judged.
    pub fn new(at: Millis, watching_since: Millis, evaluation: Evaluation, readings: Readings) -> Self {
        Self {
            at,
            watching_since,
            headline: Headline::for_findings(&evaluation.findings),
            domains: Domain::ALL.iter().map(|d| DomainSummary::of(*d, &evaluation)).collect(),
            findings: evaluation.findings,
            checks: evaluation.statuses,
            readings,
        }
    }
}
