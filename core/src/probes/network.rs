//! Network interfaces, open ports and connections, and who owns them.
//! Velapor looks at *which* connections exist, never inside the traffic.

use super::Probe;
use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::net::IpAddr;
use std::time::{Duration, Instant};
use sysinfo::{Networks, ProcessRefreshKind, ProcessesToUpdate, System};

/// One network interface and its traffic.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceReading {
    /// The system's name for it, e.g. "en0" or "Wi-Fi".
    pub name: String,
    /// Its hardware address.
    pub mac: String,
    /// Its IP addresses.
    pub addresses: Vec<String>,
    /// Bytes received per second since the last reading.
    pub received_per_sec: u64,
    /// Bytes sent per second since the last reading.
    pub sent_per_sec: u64,
    /// Bytes received since the machine started.
    pub total_received: u64,
    /// Bytes sent since the machine started.
    pub total_sent: u64,
}

/// A port a program is listening on, waiting for others to connect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListeningPort {
    /// "TCP".
    pub protocol: String,
    /// The port number.
    pub port: u16,
    /// The address it listens on, e.g. "127.0.0.1" (this machine only) or "0.0.0.0" (everyone).
    pub address: String,
    /// True when other machines can reach it (not only this machine).
    pub reachable_from_network: bool,
    /// The program that owns it, when the system says.
    pub process: Option<String>,
    /// That program's process id.
    pub pid: Option<u32>,
}

impl ListeningPort {
    /// The identity of a listener: same program on the same port.
    pub fn key(&self) -> String {
        format!("{}:{}:{}", self.protocol, self.port, self.process.as_deref().unwrap_or("?"))
    }
}

/// How many connections one program has open, and to where.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessConnections {
    /// The program.
    pub process: String,
    /// Open connections.
    pub count: usize,
    /// A few of the addresses it talks to, e.g. "140.82.112.4:443".
    pub remotes: Vec<String>,
}

/// Everything the network probe sees.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkReport {
    /// Interfaces with an address or traffic.
    pub interfaces: Vec<InterfaceReading>,
    /// Bytes received per second, all interfaces together.
    pub received_per_sec: u64,
    /// Bytes sent per second, all interfaces together.
    pub sent_per_sec: u64,
    /// Ports programs are listening on.
    pub listening: Vec<ListeningPort>,
    /// Open connections to other machines.
    pub established: usize,
    /// The programs with the most open connections.
    pub busiest: Vec<ProcessConnections>,
}

/// Reads interfaces, listening ports and connections.
pub struct NetworkProbe {
    networks: Option<Networks>,
    processes: Option<System>,
    last_read: Instant,
}

impl NetworkProbe {
    /// Finds the interfaces.
    pub fn new() -> Self {
        Self {
            networks: Networks::new_with_refreshed_list().ok(),
            processes: System::new().ok(),
            last_read: Instant::now(),
        }
    }

    /// Reads traffic per interface.
    fn interfaces(&mut self, seconds: f64) -> Vec<InterfaceReading> {
        let Some(networks) = self.networks.as_mut() else { return Vec::new() };
        networks.refresh(true);
        let mut list: Vec<InterfaceReading> = networks
            .iter()
            .filter(|(name, _)| !Self::is_loopback(&name.to_string_lossy()))
            .map(|(name, data)| InterfaceReading {
                name: name.to_string_lossy().into_owned(),
                mac: data.mac_address().to_string(),
                addresses: data
                    .ip_networks()
                    .iter()
                    .filter(|n| !n.addr.is_loopback())
                    .map(|n| n.addr.to_string())
                    .collect(),
                received_per_sec: (data.received() as f64 / seconds) as u64,
                sent_per_sec: (data.transmitted() as f64 / seconds) as u64,
                total_received: data.total_received(),
                total_sent: data.total_transmitted(),
            })
            .filter(|i| !i.addresses.is_empty() || i.total_received > 0)
            .collect();
        list.sort_by_key(|i| std::cmp::Reverse(i.total_received + i.total_sent));
        list
    }

    /// Loopback interfaces only carry traffic inside the machine.
    fn is_loopback(name: &str) -> bool {
        name == "lo" || name.starts_with("lo0") || name.starts_with("Loopback")
    }

    /// The name of each running program, by process id.
    fn process_names(&mut self) -> HashMap<u32, String> {
        let Some(system) = self.processes.as_mut() else { return HashMap::new() };
        system.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
        system.processes().iter().map(|(pid, p)| (pid.as_u32(), p.name().to_string_lossy().into_owned())).collect()
    }

    /// Reads listening ports and open connections.
    fn sockets(&mut self) -> (Vec<ListeningPort>, usize, Vec<ProcessConnections>) {
        let flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
        let Ok(sockets) = get_sockets_info(flags, ProtocolFlags::TCP) else { return (Vec::new(), 0, Vec::new()) };
        let names = self.process_names();
        let name_of = |pids: &[u32]| pids.first().and_then(|pid| names.get(pid).cloned());

        let mut listening: BTreeMap<(u16, String), ListeningPort> = BTreeMap::new();
        let mut by_process: HashMap<String, ProcessConnections> = HashMap::new();
        let mut established = 0;
        for socket in sockets {
            let ProtocolSocketInfo::Tcp(tcp) = &socket.protocol_socket_info else { continue };
            let process = name_of(&socket.associated_pids);
            match tcp.state {
                TcpState::Listen => {
                    let reachable = !tcp.local_addr.is_loopback();
                    let entry =
                        listening.entry((tcp.local_port, process.clone().unwrap_or_default())).or_insert_with(|| {
                            ListeningPort {
                                protocol: "TCP".into(),
                                port: tcp.local_port,
                                address: Self::address(tcp.local_addr),
                                reachable_from_network: reachable,
                                process: process.clone(),
                                pid: socket.associated_pids.first().copied(),
                            }
                        });
                    // Listening on IPv4 and IPv6 shows up twice: keep the widest reach.
                    if reachable && !entry.reachable_from_network {
                        entry.reachable_from_network = true;
                        entry.address = Self::address(tcp.local_addr);
                    }
                }
                TcpState::Established if !tcp.remote_addr.is_loopback() => {
                    established += 1;
                    let name = process.unwrap_or_else(|| "Unknown program".into());
                    let entry = by_process.entry(name.clone()).or_insert_with(|| ProcessConnections {
                        process: name,
                        count: 0,
                        remotes: Vec::new(),
                    });
                    entry.count += 1;
                    let remote = format!("{}:{}", Self::address(tcp.remote_addr), tcp.remote_port);
                    if entry.remotes.len() < 3 && !entry.remotes.contains(&remote) {
                        entry.remotes.push(remote);
                    }
                }
                _ => {}
            }
        }
        let mut busiest: Vec<ProcessConnections> = by_process.into_values().collect();
        busiest.sort_by(|a, b| b.count.cmp(&a.count).then(a.process.cmp(&b.process)));
        busiest.truncate(8);
        (listening.into_values().collect(), established, busiest)
    }

    /// An address as text, with "every address" written the usual way.
    fn address(addr: IpAddr) -> String {
        if addr.is_unspecified() {
            if addr.is_ipv4() {
                "0.0.0.0".into()
            } else {
                "::".into()
            }
        } else {
            addr.to_string()
        }
    }
}

impl Default for NetworkProbe {
    /// Same as [`NetworkProbe::new`].
    fn default() -> Self {
        Self::new()
    }
}

impl Probe for NetworkProbe {
    type Report = NetworkReport;

    /// Traffic is read on every tick, so the graphs move.
    fn interval(&self) -> Duration {
        Duration::ZERO
    }

    /// Reads interfaces, ports and connections.
    fn read(&mut self) -> NetworkReport {
        let seconds = self.last_read.elapsed().as_secs_f64().max(0.5);
        self.last_read = Instant::now();
        let interfaces = self.interfaces(seconds);
        let (listening, established, busiest) = self.sockets();
        NetworkReport {
            received_per_sec: interfaces.iter().map(|i| i.received_per_sec).sum(),
            sent_per_sec: interfaces.iter().map(|i| i.sent_per_sec).sum(),
            interfaces,
            listening,
            established,
            busiest,
        }
    }
}
