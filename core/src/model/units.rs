//! Numbers written the way people read them, for findings and evidence.

/// Bytes as "512 KB", "18.4 GB" (powers of 1000, like the operating systems show disks).
pub fn bytes(value: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = value as f64;
    let mut unit = 0;
    while size >= 1000.0 && unit < UNITS.len() - 1 {
        size /= 1000.0;
        unit += 1;
    }
    if unit == 0 || size >= 100.0 {
        format!("{size:.0} {}", UNITS[unit])
    } else {
        format!("{size:.1} {}", UNITS[unit])
    }
}

/// A share as "42 %".
pub fn percent(value: f64) -> String {
    format!("{value:.0} %")
}

/// A duration in seconds as "3 d 4 h", "5 h 12 min" or "40 s".
pub fn duration(seconds: u64) -> String {
    let (days, hours, minutes) = (seconds / 86_400, seconds % 86_400 / 3_600, seconds % 3_600 / 60);
    if days > 0 {
        format!("{days} d {hours} h")
    } else if hours > 0 {
        format!("{hours} h {minutes} min")
    } else if minutes > 0 {
        format!("{minutes} min")
    } else {
        format!("{seconds} s")
    }
}

/// "1 item", "3 items".
pub fn plural(count: usize, word: &str) -> String {
    if count == 1 {
        format!("1 {word}")
    } else {
        format!("{count} {word}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_numbers_for_people() {
        assert_eq!(bytes(512), "512 B");
        assert_eq!(bytes(18_400_000_000), "18.4 GB");
        assert_eq!(bytes(512_000_000_000), "512 GB");
        assert_eq!(duration(3 * 86_400 + 4 * 3_600), "3 d 4 h");
        assert_eq!(duration(40), "40 s");
        assert_eq!(plural(1, "port"), "1 port");
        assert_eq!(plural(2, "port"), "2 ports");
    }
}
