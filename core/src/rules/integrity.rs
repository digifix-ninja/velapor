//! Rules for the Integrity tab: security settings and the parts of the system that should
//! stay still.

use super::{Context, Outcome, Rule};
use crate::probes::PostureState;
use crate::{Domain, Finding, Severity};

/// The integrity rules.
pub fn rules() -> Vec<Box<dyn Rule>> {
    vec![Box::new(Posture), Box::new(ChangedFingerprint), Box::new(NewStartupItem)]
}

/// Warns when a protective setting is off.
pub struct Posture;

impl Posture {
    /// How serious it is when this setting is off.
    fn severity(key: &str) -> Severity {
        match key {
            "system-protection" | "antivirus" => Severity::Critical,
            "firewall" | "app-verification" => Severity::Warning,
            _ => Severity::Notice,
        }
    }

    /// What to say about a setting that is off.
    fn advice(key: &str) -> (&'static str, &'static str) {
        match key {
            "firewall" => (
                "The firewall decides which incoming connections are allowed. Without it, every open port is reachable.",
                "Turn the firewall on in the system's security settings.",
            ),
            "disk-encryption" => (
                "Without disk encryption, anyone holding the drive can read it, for example after a theft.",
                "Turn on disk encryption in the system's security settings, and keep the recovery key somewhere safe.",
            ),
            "system-protection" => (
                "System Integrity Protection stops anything, even with an administrator password, from changing core system files.",
                "Turn it back on from Recovery mode unless you switched it off on purpose for development.",
            ),
            "app-verification" => (
                "Gatekeeper checks that apps come from known developers before they open.",
                "Turn it back on in Privacy & Security.",
            ),
            "antivirus" => (
                "Real-time protection scans files as they arrive. Without it, harmful files are not caught.",
                "Turn real-time protection back on in Windows Security.",
            ),
            "secure-boot" => (
                "Secure Boot makes sure only signed software starts the machine.",
                "Secure Boot is switched on in the firmware settings, if your system supports it.",
            ),
            _ => ("This protection is switched off.", "Turn it back on in the system's security settings."),
        }
    }
}

impl Rule for Posture {
    /// Stable identifier.
    fn id(&self) -> &'static str {
        "integrity.posture"
    }

    /// Belongs to Integrity.
    fn domain(&self) -> Domain {
        Domain::Integrity
    }

    /// Name in the list of checks.
    fn name(&self) -> &'static str {
        "Security settings"
    }

    /// One finding for every setting that is off. Settings that couldn't be read are not
    /// reported as off.
    fn check(&self, context: &Context) -> Outcome {
        let checks = &context.readings.posture;
        if checks.is_empty() {
            return Outcome::NotApplicable;
        }
        let findings = checks
            .iter()
            .filter(|c| c.state == PostureState::Off)
            .map(|check| {
                let (explanation, recommendation) = Self::advice(&check.key);
                Finding::new(
                    format!("{}:{}", self.id(), check.key),
                    self.domain(),
                    Self::severity(&check.key),
                    format!("{} is off", check.name),
                )
                .explain(explanation)
                .recommend(recommendation)
                .evidence(&check.name, &check.detail)
            })
            .collect();
        Outcome::from_findings(findings)
    }
}

/// Warns when a watched file or setting changed since Velapor started.
pub struct ChangedFingerprint;

impl Rule for ChangedFingerprint {
    /// Stable identifier.
    fn id(&self) -> &'static str {
        "integrity.changed"
    }

    /// Belongs to Integrity.
    fn domain(&self) -> Domain {
        Domain::Integrity
    }

    /// Name in the list of checks.
    fn name(&self) -> &'static str {
        "Hosts file and DNS servers"
    }

    /// A warning when the hosts file changed, a notice when the DNS servers changed (they
    /// also change when you join another network).
    fn check(&self, context: &Context) -> Outcome {
        let fingerprints = &context.readings.fingerprints;
        if fingerprints.is_empty() {
            return Outcome::NotApplicable;
        }
        let findings = fingerprints
            .iter()
            .filter_map(|now| context.baseline.fingerprint(&now.key).filter(|then| *then != now.hash).map(|then| (now, then)))
            .map(|(now, then)| {
                let hosts = now.key == "hosts-file";
                let severity = if hosts { Severity::Warning } else { Severity::Notice };
                let finding = Finding::new(format!("{}:{}", self.id(), now.key), self.domain(), severity, format!("{} changed", now.name))
                    .evidence("Now", &now.summary)
                    .evidence("Fingerprint before", then)
                    .evidence("Fingerprint now", &now.hash);
                if hosts {
                    finding
                        .explain("The hosts file decides where some web addresses lead. Changing it can quietly send you to a different site.")
                        .cause("Developer tools, ad blockers and VPN software edit it; so does some unwanted software.")
                        .recommend("If you didn't install or change anything, look at the file and remove entries you don't recognise.")
                } else {
                    finding
                        .explain("The DNS servers translate web addresses. They change when you join another network or start a VPN.")
                        .recommend("If you didn't change networks, check the network settings for DNS servers you don't recognise.")
                }
            })
            .collect();
        Outcome::from_findings(findings)
    }
}

/// Notices programs that were set to start with the system after Velapor started.
pub struct NewStartupItem;

impl Rule for NewStartupItem {
    /// Stable identifier.
    fn id(&self) -> &'static str {
        "integrity.new-startup-item"
    }

    /// Belongs to Integrity.
    fn domain(&self) -> Domain {
        Domain::Integrity
    }

    /// Name in the list of checks.
    fn name(&self) -> &'static str {
        "New startup items"
    }

    /// A notice for each startup item that was not there when Velapor started.
    fn check(&self, context: &Context) -> Outcome {
        let findings = context
            .readings
            .software
            .startup_items
            .iter()
            .filter(|i| !context.baseline.knows_startup_item(&i.key()))
            .map(|item| {
                Finding::new(format!("{}:{}", self.id(), item.key()), self.domain(), Severity::Notice, format!("{} now starts with the system", item.name))
                    .explain("A program added itself to the list of things that start automatically. Installers do this; unwanted software does too.")
                    .recommend("If you just installed something, this is expected. If not, look up the name before you leave it.")
                    .evidence("Item", &item.name)
                    .evidence("Kind", &item.kind)
                    .evidence("Where", &item.location)
            })
            .collect();
        Outcome::from_findings(findings)
    }
}
