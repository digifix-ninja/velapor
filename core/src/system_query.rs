//! The one place in Velapor that may start another program.
//!
//! Some facts are only available from the operating system's own tools, for example whether
//! FileVault is on (`fdesetup status`) or what the Windows firewall reports. Those tools are
//! run here, and only if they are on the fixed list below. Every one of them only *reports*;
//! the arguments are fixed in the code that calls them, and no input from outside reaches them.

use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Programs Velapor may run. Each is a read-only reporting tool on its system.
const ALLOWED: &[&str] = &[
    // macOS
    "sysctl",
    "system_profiler",
    "csrutil",
    "spctl",
    "fdesetup",
    "/usr/libexec/ApplicationFirewall/socketfilterfw",
    "pmset",
    "scutil",
    // Linux
    "systemctl",
    "lsblk",
    "mokutil",
    // Windows
    "netsh",
    "reg",
    "powershell",
];

/// How long a tool may take before Velapor gives up on it.
const TIME_LIMIT: Duration = Duration::from_secs(8);

/// Asks a read-only system tool a question and returns its answer.
pub struct SystemQuery;

impl SystemQuery {
    /// Runs `program` with `args` and returns what it printed, or `None` when the tool is not
    /// allowed, not installed, fails, or takes too long.
    pub fn run(program: &str, args: &[&str]) -> Option<String> {
        if !ALLOWED.contains(&program) {
            log::warn!("refused to run {program}: not on the read-only list");
            return None;
        }
        let mut command = Command::new(program);
        command.args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
        Self::hide_window(&mut command);
        let mut child = command.spawn().ok()?;
        let mut stdout = child.stdout.take()?;
        let reader = thread::spawn(move || {
            let mut text = String::new();
            let _ = stdout.read_to_string(&mut text);
            text
        });
        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let text = reader.join().ok()?;
                    // Some tools (fdesetup, spctl) report through their exit code as well, so
                    // their text is kept even when the status is not zero.
                    return (status.success() || !text.trim().is_empty()).then_some(text);
                }
                Ok(None) if started.elapsed() < TIME_LIMIT => thread::sleep(Duration::from_millis(40)),
                _ => {
                    let _ = child.kill();
                    return None;
                }
            }
        }
    }

    /// Like [`SystemQuery::run`], but also returns what the tool printed as an error. A few tools
    /// (`spctl`, `csrutil` on some versions) write their answer there.
    pub fn run_all(program: &str, args: &[&str]) -> Option<String> {
        if !ALLOWED.contains(&program) {
            return None;
        }
        let mut command = Command::new(program);
        command.args(args).stdin(Stdio::null());
        Self::hide_window(&mut command);
        let output = command.output().ok()?;
        let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
        text.push_str(&String::from_utf8_lossy(&output.stderr));
        (!text.trim().is_empty()).then_some(text)
    }

    /// Runs a PowerShell expression on Windows and returns its trimmed answer.
    pub fn powershell(expression: &str) -> Option<String> {
        Self::run("powershell", &["-NoProfile", "-NonInteractive", "-Command", expression])
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty())
    }

    /// On Windows, keeps a console window from flashing up for each tool.
    #[cfg(windows)]
    fn hide_window(command: &mut Command) {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    /// Nothing to hide on macOS and Linux.
    #[cfg(not(windows))]
    fn hide_window(_command: &mut Command) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_programs_that_are_not_on_the_list() {
        assert_eq!(SystemQuery::run("rm", &["-rf", "/tmp/nothing"]), None);
        assert_eq!(SystemQuery::run_all("shutdown", &[]), None);
    }
}
