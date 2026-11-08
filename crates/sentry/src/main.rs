//! Velapor Sentry: the small background service that holds the elevated rights.
//!
//! Phase 0 goal: run one probe end to end. For now it takes a single sample
//! and prints it, so the workspace has a working binary to grow from.

use velapor_probes::{Probe, UptimeProbe};
use velapor_rules::Engine;
use velapor_store::{MemoryStore, Store};

/// Takes one uptime sample, stores it, runs the (still empty) rules and reports.
fn main() {
    let mut store = MemoryStore::default();
    let engine = Engine::new();
    let mut probe = UptimeProbe;

    match probe.sample() {
        Ok(sample) => {
            println!("velapor-sentry v{}: {:?}", env!("CARGO_PKG_VERSION"), sample.readings);
            store.record(&sample);
            let findings = engine.evaluate(std::slice::from_ref(&sample));
            println!("findings: {}", findings.len());
        }
        Err(error) => eprintln!("{}: {error}", probe.id()),
    }
    println!("ipc protocol v{}", velapor_ipc::PROTOCOL_VERSION);
}
