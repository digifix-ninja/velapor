//! Something Velapor wants to tell the person, with the evidence behind it.

use super::{Domain, Severity};
use crate::clock::Millis;
use serde::Serialize;

/// One measured fact a finding is based on, e.g. "Free space" → "18 GB of 500 GB".
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Evidence {
    /// What was measured.
    pub label: String,
    /// What it measured.
    pub value: String,
}

impl Evidence {
    /// Creates a piece of evidence from a label and a value.
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self { label: label.into(), value: value.into() }
    }
}

/// Something Velapor wants to tell the person. Every finding explains itself: what it means,
/// why it probably happens, what to do, and the measurements it is based on.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    /// Stable identifier, e.g. `hardware.disk-space:/`. The same problem keeps the same id.
    pub id: String,
    /// The tab it belongs to.
    pub domain: Domain,
    /// How serious it is.
    pub severity: Severity,
    /// A one-line headline.
    pub title: String,
    /// What this means, in plain language.
    pub explanation: String,
    /// Why it is probably happening, when that can be said.
    pub likely_cause: Option<String>,
    /// What the person can do about it.
    pub recommendation: String,
    /// The measurements behind it.
    pub evidence: Vec<Evidence>,
    /// When Velapor first saw it (filled in by the watchman).
    pub since: Millis,
}

impl Finding {
    /// Starts a finding with its id, domain, severity and headline. The rest is added with
    /// the builder methods below.
    pub fn new(id: impl Into<String>, domain: Domain, severity: Severity, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            domain,
            severity,
            title: title.into(),
            explanation: String::new(),
            likely_cause: None,
            recommendation: String::new(),
            evidence: Vec::new(),
            since: 0,
        }
    }

    /// Adds the plain-language explanation.
    pub fn explain(mut self, text: impl Into<String>) -> Self {
        self.explanation = text.into();
        self
    }

    /// Adds the most likely cause.
    pub fn cause(mut self, text: impl Into<String>) -> Self {
        self.likely_cause = Some(text.into());
        self
    }

    /// Adds the recommendation.
    pub fn recommend(mut self, text: impl Into<String>) -> Self {
        self.recommendation = text.into();
        self
    }

    /// Adds one piece of evidence.
    pub fn evidence(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
        self.evidence.push(Evidence::new(label, value));
        self
    }
}
