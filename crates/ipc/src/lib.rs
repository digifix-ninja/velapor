//! The local, versioned protocol between the Velapor app and Velapor Sentry.
//!
//! Transport: a Unix domain socket on macOS and Linux, a named pipe on Windows.
//! Never a network port. The API is read-mostly by design.

/// The protocol version. Bumped whenever a message changes shape.
pub const PROTOCOL_VERSION: u32 = 0;

/// Where Sentry listens on Unix-like systems.
#[cfg(unix)]
pub const SOCKET_PATH: &str = "/var/run/velapor/sentry.sock";

/// Where Sentry listens on Windows.
#[cfg(windows)]
pub const PIPE_NAME: &str = r"\\.\pipe\velapor-sentry";

/// What the app may ask Sentry. Nothing here changes the system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// Checks that Sentry is alive and speaks the same protocol.
    Hello { version: u32 },
    /// Asks for the latest sample of a probe.
    LatestSample { probe_id: String },
    /// Asks for the current findings.
    Findings,
    /// Runs every check now and returns one report.
    RunPreflight,
}
