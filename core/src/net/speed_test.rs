//! The network speed test. Runs only when the person presses the button.
//!
//! It measures latency, then download and upload speed against Cloudflare's public speed
//! test endpoints. The upload sends meaningless zero bytes; nothing about the machine is sent.

use serde::Serialize;
use std::io::Read;
use std::time::{Duration, Instant};

/// Where the test downloads from and uploads to.
const DOWNLOAD_URL: &str = "https://speed.cloudflare.com/__down";
/// Where the test uploads to.
const UPLOAD_URL: &str = "https://speed.cloudflare.com/__up";
/// How many bytes the download phase fetches.
const DOWNLOAD_BYTES: u64 = 25_000_000;
/// How many bytes the upload phase sends.
const UPLOAD_BYTES: u64 = 8_000_000;
/// How many round trips the latency phase measures.
const PINGS: usize = 8;

/// Which part of the test is running.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SpeedPhase {
    /// Measuring round-trip time.
    Latency,
    /// Measuring download speed.
    Download,
    /// Measuring upload speed.
    Upload,
}

/// A progress report while the test runs.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeedProgress {
    /// The running phase.
    pub phase: SpeedPhase,
    /// How far the phase is, 0–1.
    pub fraction: f32,
    /// The speed so far in megabits per second (latency in milliseconds for that phase).
    pub value: f64,
}

/// The finished measurement.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeedResult {
    /// Median round-trip time in milliseconds.
    pub latency_ms: f64,
    /// How much the round-trip time varies, in milliseconds.
    pub jitter_ms: f64,
    /// Download speed in megabits per second.
    pub download_mbps: f64,
    /// Upload speed in megabits per second.
    pub upload_mbps: f64,
    /// Who answered.
    pub server: String,
}

/// Runs a speed test.
pub struct SpeedTest {
    agent: ureq::Agent,
}

impl SpeedTest {
    /// Prepares a speed test with sensible time limits.
    pub fn new() -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_connect(Some(Duration::from_secs(8)))
            .timeout_global(Some(Duration::from_secs(60)))
            .user_agent("Velapor speed test")
            .build()
            .into();
        Self { agent }
    }

    /// Runs all three phases and calls `progress` along the way.
    pub fn run(&self, mut progress: impl FnMut(SpeedProgress)) -> Result<SpeedResult, String> {
        let (latency_ms, jitter_ms) = self.latency(&mut progress)?;
        let download_mbps = self.download(&mut progress)?;
        let upload_mbps = self.upload(&mut progress)?;
        Ok(SpeedResult {
            latency_ms,
            jitter_ms,
            download_mbps,
            upload_mbps,
            server: "speed.cloudflare.com".to_string(),
        })
    }

    /// Times several empty requests and returns the median and the jitter.
    fn latency(&self, progress: &mut impl FnMut(SpeedProgress)) -> Result<(f64, f64), String> {
        let mut times = Vec::with_capacity(PINGS);
        for round in 0..PINGS {
            let started = Instant::now();
            let mut response = self.agent.get(format!("{DOWNLOAD_URL}?bytes=0")).call().map_err(describe)?;
            let _ = response.body_mut().read_to_vec();
            let millis = started.elapsed().as_secs_f64() * 1000.0;
            times.push(millis);
            progress(SpeedProgress {
                phase: SpeedPhase::Latency,
                fraction: (round + 1) as f32 / PINGS as f32,
                value: millis,
            });
        }
        Ok(Self::median_and_jitter(times))
    }

    /// Downloads a fixed number of bytes and measures how fast they arrive.
    fn download(&self, progress: &mut impl FnMut(SpeedProgress)) -> Result<f64, String> {
        let mut response = self.agent.get(format!("{DOWNLOAD_URL}?bytes={DOWNLOAD_BYTES}")).call().map_err(describe)?;
        let mut reader = response.body_mut().with_config().limit(DOWNLOAD_BYTES + 1024).reader();
        let mut buffer = vec![0u8; 64 * 1024];
        let mut received = 0u64;
        let started = Instant::now();
        loop {
            let count = reader.read(&mut buffer).map_err(|e| e.to_string())?;
            if count == 0 {
                break;
            }
            received += count as u64;
            progress(SpeedProgress {
                phase: SpeedPhase::Download,
                fraction: (received as f32 / DOWNLOAD_BYTES as f32).min(1.0),
                value: Self::mbps(received, started.elapsed()),
            });
        }
        Ok(Self::mbps(received, started.elapsed()))
    }

    /// Sends a fixed number of zero bytes and measures how fast they leave.
    fn upload(&self, progress: &mut impl FnMut(SpeedProgress)) -> Result<f64, String> {
        let started = Instant::now();
        let mut zeros = Zeros::new(UPLOAD_BYTES, |sent, elapsed| {
            progress(SpeedProgress {
                phase: SpeedPhase::Upload,
                fraction: (sent as f32 / UPLOAD_BYTES as f32).min(1.0),
                value: Self::mbps(sent, elapsed),
            });
        });
        self.agent
            .post(UPLOAD_URL)
            .header("Content-Type", "application/octet-stream")
            .send(ureq::SendBody::from_reader(&mut zeros))
            .map_err(describe)?;
        Ok(Self::mbps(UPLOAD_BYTES, started.elapsed()))
    }

    /// Megabits per second for `bytes` moved in `elapsed`.
    fn mbps(bytes: u64, elapsed: Duration) -> f64 {
        let seconds = elapsed.as_secs_f64().max(0.001);
        bytes as f64 * 8.0 / seconds / 1_000_000.0
    }

    /// The median of the round trips and the average difference between neighbours.
    fn median_and_jitter(mut times: Vec<f64>) -> (f64, f64) {
        if times.is_empty() {
            return (0.0, 0.0);
        }
        let jitter = if times.len() > 1 {
            times.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f64>() / (times.len() - 1) as f64
        } else {
            0.0
        };
        times.sort_by(f64::total_cmp);
        (times[times.len() / 2], jitter)
    }
}

impl Default for SpeedTest {
    /// Same as [`SpeedTest::new`].
    fn default() -> Self {
        Self::new()
    }
}

/// Turns a network error into a sentence for the window.
fn describe(error: ureq::Error) -> String {
    format!("The speed test server could not be reached ({error})")
}

/// A reader that hands out zero bytes up to a limit and reports how many left so far.
struct Zeros<F: FnMut(u64, Duration)> {
    remaining: u64,
    sent: u64,
    started: Instant,
    report: F,
}

impl<F: FnMut(u64, Duration)> Zeros<F> {
    /// Creates a reader of `total` zero bytes.
    fn new(total: u64, report: F) -> Self {
        Self { remaining: total, sent: 0, started: Instant::now(), report }
    }
}

impl<F: FnMut(u64, Duration)> Read for Zeros<F> {
    /// Fills the buffer with zeros until the limit is reached.
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let count = (buffer.len() as u64).min(self.remaining) as usize;
        buffer[..count].fill(0);
        self.remaining -= count as u64;
        self.sent += count as u64;
        if count > 0 {
            (self.report)(self.sent, self.started.elapsed());
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// The median ignores one slow round trip; the jitter notices it.
    fn median_and_jitter() {
        let (median, jitter) = SpeedTest::median_and_jitter(vec![10.0, 12.0, 50.0, 11.0]);
        assert_eq!(median, 12.0);
        assert!(jitter > 10.0);
    }

    #[test]
    /// Eight megabits in one second is eight megabits per second.
    fn megabits_per_second() {
        assert_eq!(SpeedTest::mbps(1_000_000, Duration::from_secs(1)), 8.0);
    }
}
