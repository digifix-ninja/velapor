//! # Velapor core
//!
//! The watchman behind Velapor. It reads the machine (hardware, software, network,
//! performance and integrity), turns what it sees into findings a person can act on, and
//! never changes anything.
//!
//! How it is organised:
//!
//! * [`model`]   – the shared words: severities, domains and findings
//! * [`probes`]  – collectors that each read one part of the machine
//! * [`rules`]   – checks that turn readings into findings
//! * [`watch`]   – the [`Watchman`] that runs probes and rules, keeps a short history and
//!   remembers what the machine looked like when Velapor started
//! * [`net`]     – the two things that reach the internet, and only when asked: the speed test
//!   and the reachability checks of Preflight
//! * [`system_query`] – the one place that may start another program, limited to a fixed list
//!   of read-only system tools
//!
//! **Read-only by design.** Nothing in this crate writes a file, changes a setting or stops a
//! process. The test in `tests/read_only.rs` keeps it that way.

#![forbid(unsafe_code)]

pub mod clock;
pub mod model;
pub mod net;
pub mod probes;
pub mod rules;
pub mod system_query;
pub mod watch;

pub use model::{Domain, Evidence, Finding, Severity};
pub use net::speed_test::{SpeedProgress, SpeedResult, SpeedTest};
pub use watch::preflight::PreflightReport;
pub use watch::snapshot::Snapshot;
pub use watch::{Tick, Watchman};
