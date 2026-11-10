//! Runs the watchman on the machine the tests run on and checks that a snapshot comes out
//! whole: every tab is summarised, every rule reported, and it all turns into JSON.

use velapor_core::{Domain, Watchman};

#[test]
/// Two beats produce a complete snapshot and a history of two samples.
fn a_tick_produces_a_complete_snapshot() {
    let mut watchman = Watchman::new();
    watchman.tick();
    let tick = watchman.tick();
    let snapshot = tick.snapshot;
    assert_eq!(snapshot.domains.len(), Domain::ALL.len());
    assert!(!snapshot.checks.is_empty());
    assert!(snapshot.readings.processor.memory.total > 0);
    assert_eq!(watchman.history().len(), 2);
    let json = serde_json::to_string(&snapshot).expect("the snapshot turns into JSON");
    assert!(json.contains("\"headline\""));
}

#[test]
/// Nothing that was there at start counts as new.
fn the_start_is_the_baseline() {
    let mut watchman = Watchman::new();
    let tick = watchman.tick();
    let new_things: Vec<_> = tick
        .snapshot
        .findings
        .iter()
        .filter(|f| {
            f.id.starts_with("hardware.new-device")
                || f.id.starts_with("network.new-listener")
                || f.id.starts_with("integrity.new-startup-item")
        })
        .collect();
    assert!(new_things.is_empty(), "{new_things:?}");
}
