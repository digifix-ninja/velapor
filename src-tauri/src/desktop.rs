//! Asking the operating system to open a link in the default browser: `open` on macOS,
//! the URL handler on Windows, `xdg-open` on Linux. Only links that `About::may_open`
//! allows ever reach this.

use std::process::Command;

/// The operating system's own helpers.
pub struct Desktop;

impl Desktop {
    /// Opens a web page in the default browser.
    pub fn open_url(url: &str) -> Result<(), String> {
        Self::open_command(url).status().map(|_| ()).map_err(|e| format!("Couldn't open the browser: {e}"))
    }

    /// macOS: `open <url>`.
    #[cfg(target_os = "macos")]
    fn open_command(url: &str) -> Command {
        let mut command = Command::new("open");
        command.arg(url);
        command
    }

    /// Windows: hands the link to the default handler without going through a shell.
    #[cfg(windows)]
    fn open_command(url: &str) -> Command {
        let mut command = Command::new("rundll32");
        command.args(["url.dll,FileProtocolHandler", url]);
        command
    }

    /// Linux: `xdg-open <url>`.
    #[cfg(all(unix, not(target_os = "macos")))]
    fn open_command(url: &str) -> Command {
        let mut command = Command::new("xdg-open");
        command.arg(url);
        command
    }
}
