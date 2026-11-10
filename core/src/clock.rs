//! Time as Velapor keeps it: milliseconds since the Unix epoch, easy to send to the window.

use std::time::{SystemTime, UNIX_EPOCH};

/// A moment in time, in milliseconds since 1 January 1970 (UTC).
pub type Millis = u64;

/// The current time.
pub fn now() -> Millis {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as Millis).unwrap_or(0)
}
