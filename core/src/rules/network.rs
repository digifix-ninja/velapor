//! Rules for the Network tab: open ports and connectivity.

use super::{Context, Outcome, Rule};
use crate::probes::ListeningPort;
use crate::{Domain, Finding, Severity};

/// The network rules.
pub fn rules() -> Vec<Box<dyn Rule>> {
    vec![Box::new(NewListener), Box::new(RiskyPort), Box::new(Offline)]
}

/// Ports that are often attacked when other machines on the network can reach them.
const RISKY_PORTS: &[(u16, &str)] = &[
    (21, "FTP"),
    (23, "Telnet"),
    (135, "Windows RPC"),
    (139, "NetBIOS"),
    (445, "Windows file sharing (SMB)"),
    (3389, "Remote Desktop"),
    (5900, "Screen sharing (VNC)"),
];

/// The well-known service behind a risky port, if it is one.
fn risky_service(port: &ListeningPort) -> Option<&'static str> {
    RISKY_PORTS.iter().find(|(number, _)| *number == port.port).map(|(_, name)| *name)
}

/// The name of the program behind a port, or a plain fallback.
fn owner(port: &ListeningPort) -> String {
    port.process.clone().unwrap_or_else(|| "An unknown program".to_string())
}

/// Notices programs that started listening for connections after Velapor started.
pub struct NewListener;

impl Rule for NewListener {
    /// Stable identifier.
    fn id(&self) -> &'static str {
        "network.new-listener"
    }

    /// Belongs to Network.
    fn domain(&self) -> Domain {
        Domain::Network
    }

    /// Name in the list of checks.
    fn name(&self) -> &'static str {
        "New open ports"
    }

    /// A notice for a new port only this machine can reach, a warning when other machines
    /// on the network can reach it too.
    fn check(&self, context: &Context) -> Outcome {
        let findings = context
            .readings
            .network
            .listening
            .iter()
            .filter(|p| !context.baseline.knows_listening_port(&p.key()))
            .map(|port| {
                let severity = if port.reachable_from_network { Severity::Warning } else { Severity::Notice };
                let reach = if port.reachable_from_network { "other machines on the network" } else { "this machine only" };
                Finding::new(format!("{}:{}", self.id(), port.key()), self.domain(), severity, format!("{} opened port {}", owner(port), port.port))
                    .explain("A program started waiting for incoming connections. Developer tools and sharing features do this; unwanted software does too.")
                    .recommend("If you recognise the program, this is fine. If not, find out what it is before you leave it running.")
                    .evidence("Port", format!("{} {}", port.protocol, port.port))
                    .evidence("Listening on", &port.address)
                    .evidence("Reachable from", reach)
            })
            .collect();
        Outcome::from_findings(findings)
    }
}

/// Warns about well-known risky services that other machines can reach.
pub struct RiskyPort;

impl Rule for RiskyPort {
    /// Stable identifier.
    fn id(&self) -> &'static str {
        "network.risky-port"
    }

    /// Belongs to Network.
    fn domain(&self) -> Domain {
        Domain::Network
    }

    /// Name in the list of checks.
    fn name(&self) -> &'static str {
        "Exposed services"
    }

    /// A warning for each risky service reachable from the network.
    fn check(&self, context: &Context) -> Outcome {
        let findings = context
            .readings
            .network
            .listening
            .iter()
            .filter(|p| p.reachable_from_network)
            .filter_map(|port| risky_service(port).map(|service| (port, service)))
            .map(|(port, service)| {
                Finding::new(format!("{}:{}", self.id(), port.port), self.domain(), Severity::Warning, format!("{service} is open to the network"))
                    .explain("Other machines on this network can reach this service. It is a common target for attacks, especially on public Wi-Fi.")
                    .cause(format!("{} is listening on port {}.", owner(port), port.port))
                    .recommend("Turn the service off in the system settings if you don't use it, and keep the firewall on.")
                    .evidence("Port", format!("{} {}", port.protocol, port.port))
                    .evidence("Listening on", &port.address)
            })
            .collect();
        Outcome::from_findings(findings)
    }
}

/// Notices when no network interface has an address.
pub struct Offline;

impl Rule for Offline {
    /// Stable identifier.
    fn id(&self) -> &'static str {
        "network.offline"
    }

    /// Belongs to Network.
    fn domain(&self) -> Domain {
        Domain::Network
    }

    /// Name in the list of checks.
    fn name(&self) -> &'static str {
        "Network connection"
    }

    /// A notice when no interface except loopback has an address.
    fn check(&self, context: &Context) -> Outcome {
        let interfaces = &context.readings.network.interfaces;
        if interfaces.is_empty() {
            return Outcome::NotApplicable;
        }
        let connected = interfaces.iter().any(|i| {
            i.addresses
                .iter()
                .any(|a| !a.starts_with("127.") && a != "::1" && !a.starts_with("fe80") && !a.starts_with("169.254"))
        });
        Outcome::from_option((!connected).then(|| {
            Finding::new(self.id(), self.domain(), Severity::Notice, "Not connected to a network")
                .explain("No network interface has an address, so nothing can reach the internet.")
                .recommend("Check Wi-Fi or the cable. Preflight can test the connection once it is back.")
        }))
    }
}
