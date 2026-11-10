//! Everything attached to the machine: USB devices, Bluetooth devices and the power adapter.
//! (Displays are added by the app, which already knows them from the window system.)

use super::Probe;
#[allow(unused_imports)]
use crate::system_query::SystemQuery;
use nusb::MaybeFuture;
use serde::Serialize;
use std::time::Duration;

/// One device attached to the machine.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachedDevice {
    /// Identity that stays the same while the device stays attached, e.g. "usb:05ac:0265".
    pub key: String,
    /// What the device calls itself, e.g. "Magic Keyboard with Touch ID".
    pub name: String,
    /// Its maker, when it says.
    pub maker: Option<String>,
    /// "USB", "Bluetooth", "Power" or "Display".
    pub kind: String,
    /// How it is connected, e.g. "USB 3 · 10 Gb/s".
    pub connection: String,
    /// A short extra detail, e.g. "Keyboard" or "96 W".
    pub detail: String,
}

/// Lists attached devices.
pub struct DeviceProbe;

impl DeviceProbe {
    /// USB devices, without the controllers built into the machine.
    fn usb() -> Vec<AttachedDevice> {
        let Ok(devices) = nusb::list_devices().wait() else { return Vec::new() };
        devices
            .filter(|d| d.vendor_id() != 0x1d6b) // Linux's own root hubs
            .map(|d| {
                let name = d
                    .product_string()
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("USB device {:04x}:{:04x}", d.vendor_id(), d.product_id()));
                AttachedDevice {
                    key: format!("usb:{:04x}:{:04x}:{}", d.vendor_id(), d.product_id(), d.bus_id()),
                    name,
                    maker: d.manufacturer_string().map(str::to_string),
                    kind: "USB".into(),
                    connection: Self::usb_speed(d.speed()),
                    detail: Self::usb_class(d.class()).into(),
                }
            })
            .collect()
    }

    /// The USB generation and speed in words.
    fn usb_speed(speed: Option<nusb::Speed>) -> String {
        match speed {
            Some(nusb::Speed::Low) => "USB 1 · 1.5 Mb/s".into(),
            Some(nusb::Speed::Full) => "USB 1 · 12 Mb/s".into(),
            Some(nusb::Speed::High) => "USB 2 · 480 Mb/s".into(),
            Some(nusb::Speed::Super) => "USB 3 · 5 Gb/s".into(),
            Some(nusb::Speed::SuperPlus) => "USB 3 · 10 Gb/s".into(),
            Some(_) => "USB · 20 Gb/s or faster".into(),
            None => "USB".into(),
        }
    }

    /// What kind of device a USB class code stands for (0 means "see its interfaces").
    fn usb_class(class: u8) -> &'static str {
        match class {
            0x01 => "Audio",
            0x02 | 0x0a => "Communication",
            0x03 => "Input device",
            0x06 => "Camera or scanner",
            0x07 => "Printer",
            0x08 => "Storage",
            0x09 => "Hub",
            0x0e => "Video",
            0x0b => "Smart card",
            0xe0 => "Wireless",
            0x11 => "Billboard",
            _ => "Device",
        }
    }

    /// Connected Bluetooth devices, from macOS's system report.
    #[cfg(target_os = "macos")]
    fn bluetooth() -> Vec<AttachedDevice> {
        let Some(text) = SystemQuery::run("system_profiler", &["SPBluetoothDataType"]) else { return Vec::new() };
        BluetoothReport::parse(&text)
    }

    /// Bluetooth needs root or a desktop service on Linux and Windows; not read yet.
    #[cfg(not(target_os = "macos"))]
    fn bluetooth() -> Vec<AttachedDevice> {
        Vec::new()
    }

    /// The power adapter on macOS, with its wattage.
    #[cfg(target_os = "macos")]
    fn power() -> Vec<AttachedDevice> {
        let Some(text) = SystemQuery::run("pmset", &["-g", "adapter"]) else { return Vec::new() };
        let field = |key: &str| {
            text.lines()
                .find_map(|l| l.trim().strip_prefix(key).map(|v| v.trim_start_matches([' ', '=']).trim().to_string()))
        };
        let Some(watts) = field("Wattage") else { return Vec::new() };
        vec![AttachedDevice {
            key: "power:adapter".into(),
            name: field("Name").unwrap_or_else(|| "Power adapter".into()),
            maker: field("Manufacturer"),
            kind: "Power".into(),
            connection: "Charging port".into(),
            detail: watts.replace('W', " W"),
        }]
    }

    /// The power adapter on Linux, from the power supply list every user can read.
    #[cfg(target_os = "linux")]
    fn power() -> Vec<AttachedDevice> {
        let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") else { return Vec::new() };
        let read = |path: &std::path::Path, name: &str| {
            std::fs::read_to_string(path.join(name)).unwrap_or_default().trim().to_string()
        };
        entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| read(p, "type") == "Mains" && read(p, "online") == "1")
            .take(1)
            .map(|_| AttachedDevice {
                key: "power:adapter".into(),
                name: "Power adapter".into(),
                maker: None,
                kind: "Power".into(),
                connection: "Mains".into(),
                detail: "Connected".into(),
            })
            .collect()
    }

    /// Windows reports the adapter through the battery's charging state instead.
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    fn power() -> Vec<AttachedDevice> {
        Vec::new()
    }
}

impl Probe for DeviceProbe {
    type Report = Vec<AttachedDevice>;

    /// Devices come and go; every few seconds notices a new one quickly enough.
    fn interval(&self) -> Duration {
        Duration::from_secs(6)
    }

    /// Lists power, USB and Bluetooth devices, in that order.
    fn read(&mut self) -> Vec<AttachedDevice> {
        let mut devices = Self::power();
        devices.extend(Self::usb());
        devices.extend(Self::bluetooth());
        devices
    }
}

/// Reads the "Connected:" part of macOS's Bluetooth report.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
struct BluetoothReport;

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
impl BluetoothReport {
    /// Finds each connected device's name and its "Minor Type" (keyboard, headphones…).
    fn parse(text: &str) -> Vec<AttachedDevice> {
        let indent = |line: &str| line.len() - line.trim_start().len();
        let mut devices = Vec::new();
        let mut section: Option<usize> = None;
        let mut name_indent: Option<usize> = None;
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            let depth = indent(line);
            let trimmed = line.trim();
            if trimmed == "Connected:" {
                section = Some(depth);
                name_indent = None;
                continue;
            }
            let Some(start) = section else { continue };
            if depth <= start {
                section = None;
                continue;
            }
            if trimmed.ends_with(':') && name_indent.is_none_or(|n| depth == n) {
                name_indent = Some(depth);
                let name = trimmed.trim_end_matches(':').to_string();
                devices.push(AttachedDevice {
                    key: format!("bt:{name}"),
                    name,
                    maker: None,
                    kind: "Bluetooth".into(),
                    connection: "Bluetooth".into(),
                    detail: "Device".into(),
                });
            } else if let (Some(device), Some(kind)) = (devices.last_mut(), trimmed.strip_prefix("Minor Type:")) {
                device.detail = kind.trim().to_string();
            }
        }
        devices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_connected_bluetooth_devices() {
        let report = "Bluetooth:\n\n      Bluetooth Controller:\n          State: On\n      Connected:\n          AirPods Pro:\n              Address: 00:00\n              Minor Type: Headphones\n          Magic Keyboard:\n              Minor Type: Keyboard\n      Not Connected:\n          Old Mouse:\n              Minor Type: Mouse\n";
        let devices = BluetoothReport::parse(report);
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].name, "AirPods Pro");
        assert_eq!(devices[0].detail, "Headphones");
        assert_eq!(devices[1].detail, "Keyboard");
    }
}
