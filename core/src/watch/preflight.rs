//! Preflight: a full check on demand. Every probe is read fresh, every rule runs, and the
//! connection to the internet is tested.

use super::snapshot::Headline;
use crate::clock::{self, Millis};
use crate::net::reachability::{Reachability, ReachabilityCheck};
use crate::rules::{Evaluation, RuleStatus};
use crate::{Domain, Finding, Severity};
use serde::Serialize;

/// The two connection checks Preflight adds.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionChecks {
    /// Can names be looked up?
    pub names: Reachability,
    /// Can the internet be reached?
    pub internet: Reachability,
}

impl ConnectionChecks {
    /// Runs both checks. Takes a few seconds at most.
    pub fn run() -> Self {
        Self { names: ReachabilityCheck::name_lookup(), internet: ReachabilityCheck::internet() }
    }

    /// Findings for checks that failed.
    fn findings(&self) -> Vec<Finding> {
        let mut findings = Vec::new();
        if !self.internet.ok {
            findings.push(
                Finding::new("network.internet", Domain::Network, Severity::Warning, "The internet can't be reached")
                    .explain("A direct connection to well-known public addresses failed.")
                    .cause("No network, a captive portal (hotel or café Wi-Fi) that wants you to sign in, or a blocking firewall.")
                    .recommend("Open a web page to see whether the network asks you to sign in, or check the router.")
                    .evidence("Result", &self.internet.detail),
            );
        } else if !self.names.ok {
            findings.push(
                Finding::new("network.dns", Domain::Network, Severity::Warning, "Web addresses can't be looked up")
                    .explain("The internet answers, but names like example.com can't be turned into addresses, so most things won't load.")
                    .cause("The DNS servers set for this network don't answer.")
                    .recommend("Reconnect to the network, or check the DNS servers in the network settings.")
                    .evidence("Result", &self.names.detail),
            );
        }
        findings
    }
}

/// The result of a Preflight run.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreflightReport {
    /// When it finished.
    pub at: Millis,
    /// How long it took, in milliseconds.
    pub took_ms: u64,
    /// The verdict.
    pub verdict: Headline,
    /// How each check came out.
    pub checks: Vec<RuleStatus>,
    /// The connection checks.
    pub connection: ConnectionChecks,
    /// Every finding, worst first.
    pub findings: Vec<Finding>,
}

impl PreflightReport {
    /// Puts the report together from a fresh evaluation and the connection checks.
    pub fn new(started: Millis, evaluation: Evaluation, connection: ConnectionChecks) -> Self {
        let mut findings = evaluation.findings;
        findings.extend(connection.findings());
        findings.sort_by_key(|f| std::cmp::Reverse(f.severity));
        let at = clock::now();
        Self {
            at,
            took_ms: at.saturating_sub(started),
            verdict: Headline::for_findings(&findings),
            checks: evaluation.statuses,
            connection,
            findings,
        }
    }
}
