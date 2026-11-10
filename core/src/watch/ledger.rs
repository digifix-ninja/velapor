//! Remembers when each finding first appeared, so the window can say "since 14:02" and only
//! new problems raise a notification.

use crate::clock::Millis;
use crate::Finding;
use std::collections::HashMap;

/// When each open finding was first seen.
#[derive(Debug, Default)]
pub struct Ledger {
    first_seen: HashMap<String, Millis>,
}

impl Ledger {
    /// Creates an empty ledger.
    pub fn new() -> Self {
        Self::default()
    }

    /// Fills in `since` on every finding, forgets findings that have cleared, and returns
    /// the ones that are new on this tick.
    pub fn stamp(&mut self, now: Millis, findings: &mut [Finding]) -> Vec<Finding> {
        self.first_seen.retain(|id, _| findings.iter().any(|f| &f.id == id));
        let mut fresh = Vec::new();
        for finding in findings.iter_mut() {
            let seen = *self.first_seen.entry(finding.id.clone()).or_insert_with(|| {
                fresh.push(finding.id.clone());
                now
            });
            finding.since = seen;
        }
        findings.iter().filter(|f| fresh.contains(&f.id)).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Domain, Severity};

    /// A finding with only an id.
    fn finding(id: &str) -> Finding {
        Finding::new(id, Domain::Hardware, Severity::Warning, id)
    }

    #[test]
    /// A finding keeps its first time and is only "new" once.
    fn keeps_the_first_time() {
        let mut ledger = Ledger::new();
        let mut first = vec![finding("a")];
        assert_eq!(ledger.stamp(100, &mut first).len(), 1);
        let mut second = vec![finding("a"), finding("b")];
        let fresh = ledger.stamp(200, &mut second);
        assert_eq!(fresh.len(), 1);
        assert_eq!(fresh[0].id, "b");
        assert_eq!(second[0].since, 100);
        assert_eq!(second[1].since, 200);
    }

    #[test]
    /// A finding that cleared and comes back counts as new again.
    fn forgets_cleared_findings() {
        let mut ledger = Ledger::new();
        ledger.stamp(100, &mut [finding("a")]);
        ledger.stamp(200, &mut []);
        let mut again = vec![finding("a")];
        assert_eq!(ledger.stamp(300, &mut again).len(), 1);
        assert_eq!(again[0].since, 300);
    }
}
