//! Small connection checks for Preflight: can names be looked up, and can the internet be
//! reached? They open a connection and close it again; nothing is sent.

use serde::Serialize;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

/// How long one check may wait.
const TIME_LIMIT: Duration = Duration::from_secs(4);

/// The answer of one connection check.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reachability {
    /// Whether it worked.
    pub ok: bool,
    /// How long it took, in milliseconds.
    pub millis: u64,
    /// What happened, in a few words.
    pub detail: String,
}

/// Runs the connection checks.
pub struct ReachabilityCheck;

impl ReachabilityCheck {
    /// Looks up a well-known name through the system's DNS servers.
    pub fn name_lookup() -> Reachability {
        let started = Instant::now();
        let ok = ("one.one.one.one", 443).to_socket_addrs().is_ok_and(|mut addresses| addresses.next().is_some());
        let millis = started.elapsed().as_millis() as u64;
        let detail = if ok { "Names are resolved" } else { "Names could not be looked up" };
        Reachability { ok, millis, detail: detail.to_string() }
    }

    /// Opens (and at once closes) a connection to a public address, without using DNS.
    pub fn internet() -> Reachability {
        let targets: [SocketAddr; 2] = [([1, 1, 1, 1], 443).into(), ([9, 9, 9, 9], 443).into()];
        let started = Instant::now();
        let ok = targets.iter().any(|target| TcpStream::connect_timeout(target, TIME_LIMIT).is_ok());
        let millis = started.elapsed().as_millis() as u64;
        let detail = if ok { "The internet answers" } else { "The internet could not be reached" };
        Reachability { ok, millis, detail: detail.to_string() }
    }
}
