//! Every rule, judged against made-up readings: quiet when things are fine, and the right
//! severity when they are not.

use velapor_core::probes::{
    AttachedDevice, BatteryReading, DiskReading, Fingerprint, InterfaceReading, ListeningPort, PostureCheck,
    PostureState, ProcessReading, SensorReading, StartupItem,
};
use velapor_core::rules::{hardware, integrity, network, performance, software, Context, Outcome, Rule};
use velapor_core::watch::baseline::Baseline;
use velapor_core::watch::history::{History, Sample};
use velapor_core::watch::readings::Readings;
use velapor_core::Severity;

const GB: u64 = 1_000_000_000;

/// A test bench: readings to judge, the baseline captured from the "start" readings, and a history.
struct Bench {
    now: u64,
    start: Readings,
    readings: Readings,
    history: History,
}

impl Bench {
    /// A quiet machine with 16 GB of memory, whose start looks the same as now.
    fn new() -> Self {
        let mut readings = Readings::default();
        readings.processor.memory.total = 16 * GB;
        readings.processor.memory.available = 10 * GB;
        readings.processor.memory.used = 6 * GB;
        Self { now: 3_600_000, start: readings.clone(), readings, history: History::new(1800) }
    }

    /// Changes the readings now, after the baseline was taken.
    fn now(mut self, change: impl FnOnce(&mut Readings)) -> Self {
        change(&mut self.readings);
        self
    }

    /// Changes both the start and now, so the baseline already knows about it.
    fn always(mut self, change: impl Fn(&mut Readings)) -> Self {
        change(&mut self.start);
        change(&mut self.readings);
        self
    }

    /// Fills the history with a sample every two seconds for the given number of minutes.
    fn busy_for(mut self, minutes: u64, cpu: f32) -> Self {
        let mut sample = self.readings.clone();
        sample.processor.cpu.usage = cpu;
        let from = self.now - minutes * 60_000;
        for at in (from..=self.now).step_by(2_000) {
            self.history.push(Sample::from_readings(at, &sample));
        }
        self
    }

    /// Runs one rule.
    fn judge(&self, rule: &dyn Rule) -> Outcome {
        let baseline = Baseline::capture(0, &self.start);
        rule.check(&Context { now: self.now, readings: &self.readings, history: &self.history, baseline: &baseline })
    }

    /// The severities a rule found, worst first; empty when it was clear.
    fn severities(&self, rule: &dyn Rule) -> Vec<Severity> {
        match self.judge(rule) {
            Outcome::Found(findings) => {
                let mut list: Vec<Severity> = findings.iter().map(|f| f.severity).collect();
                list.sort_by(|a, b| b.cmp(a));
                list
            }
            Outcome::Clear | Outcome::NotApplicable => Vec::new(),
        }
    }
}

/// A drive with the given size and free space.
fn disk(mount: &str, total: u64, available: u64, read_only: bool) -> DiskReading {
    DiskReading {
        name: mount.into(),
        mount_point: mount.into(),
        file_system: "apfs".into(),
        kind: "SSD".into(),
        total,
        available,
        removable: false,
        read_only,
        read_per_sec: 0,
        written_per_sec: 0,
    }
}

/// A listening port.
fn port(number: u16, process: &str, reachable: bool) -> ListeningPort {
    ListeningPort {
        protocol: "TCP".into(),
        port: number,
        address: if reachable { "0.0.0.0".into() } else { "127.0.0.1".into() },
        reachable_from_network: reachable,
        process: Some(process.into()),
        pid: Some(42),
    }
}

/// A process using the given memory.
fn process(name: &str, memory: u64) -> ProcessReading {
    ProcessReading { pid: 7, name: name.into(), cpu: 99.0, memory, read_per_sec: 0, written_per_sec: 0, run_time: 60 }
}

/// A network interface with addresses.
fn interface(addresses: &[&str]) -> InterfaceReading {
    InterfaceReading {
        name: "en0".into(),
        mac: "00:00:00:00:00:00".into(),
        addresses: addresses.iter().map(|a| a.to_string()).collect(),
        received_per_sec: 0,
        sent_per_sec: 0,
        total_received: 0,
        total_sent: 0,
    }
}

/// A fingerprint of something that should stay still.
fn fingerprint(key: &str, hash: &str) -> Fingerprint {
    Fingerprint { key: key.into(), name: key.into(), hash: hash.into(), summary: String::new() }
}

/// A security setting in the given state.
fn posture(key: &str, state: PostureState) -> PostureCheck {
    PostureCheck { key: key.into(), name: key.into(), state, detail: String::new() }
}

// ------------------------------------------------------------------- hardware

#[test]
fn disk_space_grades_full_drives_and_skips_read_only_ones() {
    let bench = Bench::new().now(|r| {
        r.disks = vec![
            disk("/", 1000 * GB, 500 * GB, false),      // half full: fine
            disk("/Data", 1000 * GB, 60 * GB, false),   // 94 % full: warning
            disk("/Backup", 1000 * GB, 10 * GB, false), // 99 % full: critical
            disk("/System", 1000 * GB, 0, true),        // read-only: skipped
        ]
    });
    assert_eq!(bench.severities(&hardware::DiskSpace), vec![Severity::Critical, Severity::Warning]);
    assert_eq!(Bench::new().judge(&hardware::DiskSpace), Outcome::NotApplicable);
}

#[test]
fn temperature_warns_near_the_limit() {
    let sensor = |celsius, critical| SensorReading { label: "CPU".into(), celsius, highest: None, critical };
    let warm = Bench::new().now(|r| r.sensors = vec![sensor(70.0, None)]);
    let hot = Bench::new().now(|r| r.sensors = vec![sensor(88.0, None)]);
    let over = Bench::new().now(|r| r.sensors = vec![sensor(81.0, Some(80.0))]);
    assert_eq!(warm.judge(&hardware::Temperature), Outcome::Clear);
    assert_eq!(hot.severities(&hardware::Temperature), vec![Severity::Warning]);
    assert_eq!(over.severities(&hardware::Temperature), vec![Severity::Critical]);
}

#[test]
fn battery_health_notices_wear() {
    let battery = |health| BatteryReading {
        charge: 50.0,
        health,
        cycles: Some(400),
        state: "Discharging".into(),
        temperature: None,
        model: None,
        technology: "Li-ion".into(),
    };
    assert_eq!(Bench::new().now(|r| r.battery = Some(battery(92.0))).judge(&hardware::BatteryHealth), Outcome::Clear);
    assert_eq!(
        Bench::new().now(|r| r.battery = Some(battery(75.0))).severities(&hardware::BatteryHealth),
        vec![Severity::Notice]
    );
    assert_eq!(
        Bench::new().now(|r| r.battery = Some(battery(50.0))).severities(&hardware::BatteryHealth),
        vec![Severity::Warning]
    );
    assert_eq!(Bench::new().judge(&hardware::BatteryHealth), Outcome::NotApplicable);
}

#[test]
fn new_device_notices_only_what_was_not_there_at_start() {
    let device = |key: &str| AttachedDevice {
        key: key.into(),
        name: key.into(),
        maker: None,
        kind: "Storage".into(),
        connection: "USB".into(),
        detail: String::new(),
    };
    let bench = Bench::new().always(|r| r.devices = vec![device("keyboard")]).now(|r| r.devices.push(device("stick")));
    assert_eq!(bench.severities(&hardware::NewDevice), vec![Severity::Notice]);
}

// ------------------------------------------------------------------- software

#[test]
fn long_uptime_notices_after_thirty_days() {
    let day = 86_400;
    assert_eq!(Bench::new().now(|r| r.software.uptime = 3 * day).judge(&software::LongUptime), Outcome::Clear);
    assert_eq!(
        Bench::new().now(|r| r.software.uptime = 40 * day).severities(&software::LongUptime),
        vec![Severity::Notice]
    );
}

// ------------------------------------------------------------------- network

#[test]
fn new_listener_is_a_warning_when_the_network_can_reach_it() {
    let bench = Bench::new().always(|r| r.network.listening = vec![port(631, "cupsd", false)]).now(|r| {
        r.network.listening.push(port(8384, "syncthing", true));
        r.network.listening.push(port(5432, "postgres", false));
    });
    assert_eq!(bench.severities(&network::NewListener), vec![Severity::Warning, Severity::Notice]);
}

#[test]
fn risky_port_only_counts_when_reachable() {
    let open = Bench::new().always(|r| r.network.listening = vec![port(3389, "rdp", true)]);
    let local = Bench::new().always(|r| r.network.listening = vec![port(5900, "vnc", false)]);
    assert_eq!(open.severities(&network::RiskyPort), vec![Severity::Warning]);
    assert_eq!(local.judge(&network::RiskyPort), Outcome::Clear);
}

#[test]
fn offline_ignores_loopback_and_link_local_addresses() {
    let lonely = Bench::new().now(|r| r.network.interfaces = vec![interface(&["127.0.0.1", "fe80::1", "169.254.3.4"])]);
    let online = Bench::new().now(|r| r.network.interfaces = vec![interface(&["192.168.1.20"])]);
    assert_eq!(lonely.severities(&network::Offline), vec![Severity::Notice]);
    assert_eq!(online.judge(&network::Offline), Outcome::Clear);
}

// ------------------------------------------------------------------- performance

#[test]
fn sustained_load_needs_five_busy_minutes() {
    assert_eq!(Bench::new().busy_for(1, 100.0).judge(&performance::SustainedLoad), Outcome::Clear);
    assert_eq!(Bench::new().busy_for(6, 50.0).judge(&performance::SustainedLoad), Outcome::Clear);
    assert_eq!(Bench::new().busy_for(6, 90.0).severities(&performance::SustainedLoad), vec![Severity::Notice]);
    assert_eq!(Bench::new().busy_for(6, 99.0).severities(&performance::SustainedLoad), vec![Severity::Warning]);
}

#[test]
fn memory_pressure_grades_what_is_left() {
    let left = |available| Bench::new().now(move |r| r.processor.memory.available = available);
    assert_eq!(left(4 * GB).judge(&performance::MemoryPressure), Outcome::Clear);
    assert_eq!(left(GB).severities(&performance::MemoryPressure), vec![Severity::Warning]);
    assert_eq!(left(GB / 4).severities(&performance::MemoryPressure), vec![Severity::Critical]);
}

#[test]
fn swap_grades_how_much_is_in_use() {
    let swap = |used| {
        Bench::new().now(move |r| {
            r.processor.memory.swap_total = 4 * GB;
            r.processor.memory.swap_used = used;
        })
    };
    assert_eq!(swap(GB).judge(&performance::Swap), Outcome::Clear);
    assert_eq!(swap(3 * GB).severities(&performance::Swap), vec![Severity::Notice]);
    assert_eq!(swap(4 * GB).severities(&performance::Swap), vec![Severity::Warning]);
    assert_eq!(Bench::new().judge(&performance::Swap), Outcome::NotApplicable);
}

#[test]
fn hungry_process_notices_more_than_a_third_of_memory() {
    let bench = Bench::new().now(|r| r.performance.by_memory = vec![process("browser", 6 * GB), process("editor", GB)]);
    assert_eq!(bench.severities(&performance::HungryProcess), vec![Severity::Notice]);
}

// ------------------------------------------------------------------- integrity

#[test]
fn posture_grades_what_is_off_and_ignores_what_cannot_be_read() {
    let bench = Bench::new().now(|r| {
        r.posture = vec![
            posture("system-protection", PostureState::Off),
            posture("firewall", PostureState::Off),
            posture("disk-encryption", PostureState::On),
            posture("secure-boot", PostureState::Unknown),
        ]
    });
    assert_eq!(bench.severities(&integrity::Posture), vec![Severity::Critical, Severity::Warning]);
}

#[test]
fn changed_fingerprint_warns_about_the_hosts_file() {
    let bench = Bench::new()
        .always(|r| r.fingerprints = vec![fingerprint("hosts-file", "aaa"), fingerprint("dns-servers", "bbb")])
        .now(|r| r.fingerprints = vec![fingerprint("hosts-file", "ccc"), fingerprint("dns-servers", "ddd")]);
    assert_eq!(bench.severities(&integrity::ChangedFingerprint), vec![Severity::Warning, Severity::Notice]);
    let same = Bench::new().always(|r| r.fingerprints = vec![fingerprint("hosts-file", "aaa")]);
    assert_eq!(same.judge(&integrity::ChangedFingerprint), Outcome::Clear);
}

#[test]
fn new_startup_item_notices_only_new_ones() {
    let item = |name: &str| StartupItem {
        name: name.into(),
        kind: "Launch agent".into(),
        location: "~/Library/LaunchAgents".into(),
    };
    let bench = Bench::new()
        .always(|r| r.software.startup_items = vec![item("old")])
        .now(|r| r.software.startup_items.push(item("new")));
    assert_eq!(bench.severities(&integrity::NewStartupItem), vec![Severity::Notice]);
}
