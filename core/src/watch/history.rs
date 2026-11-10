//! A short memory of the last hour, so the window can draw trends and rules can tell a
//! moment's spike from a lasting load.

use super::readings::Readings;
use crate::clock::Millis;
use serde::Serialize;
use std::collections::VecDeque;

/// One point in time, reduced to the numbers worth drawing.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sample {
    /// When it was taken.
    pub at: Millis,
    /// Processor load, 0–100.
    pub cpu: f32,
    /// Memory in use, 0–100.
    pub memory: f32,
    /// Swap in use, 0–100.
    pub swap: f32,
    /// Bytes per second received over the network.
    pub received: u64,
    /// Bytes per second sent over the network.
    pub sent: u64,
    /// Bytes per second read from drives.
    pub disk_read: u64,
    /// Bytes per second written to drives (by any program; Velapor writes nothing).
    pub disk_written: u64,
    /// The warmest sensor in °C, if any sensor answers.
    pub temperature: Option<f32>,
}

impl Sample {
    /// Reduces a full set of readings to one sample.
    pub fn from_readings(at: Millis, readings: &Readings) -> Self {
        let memory = &readings.processor.memory;
        Self {
            at,
            cpu: readings.processor.cpu.usage,
            memory: memory.used_percent() as f32,
            swap: memory.swap_percent() as f32,
            received: readings.network.received_per_sec,
            sent: readings.network.sent_per_sec,
            disk_read: readings.performance.disk_read_per_sec,
            disk_written: readings.performance.disk_written_per_sec,
            temperature: readings.hottest_sensor().map(|s| s.celsius),
        }
    }
}

/// A ring of samples. When it is full the oldest one drops off.
#[derive(Debug, Clone)]
pub struct History {
    samples: VecDeque<Sample>,
    capacity: usize,
}

impl History {
    /// Creates an empty history that keeps at most `capacity` samples.
    pub fn new(capacity: usize) -> Self {
        Self { samples: VecDeque::with_capacity(capacity), capacity }
    }

    /// Adds a sample, forgetting the oldest if the ring is full.
    pub fn push(&mut self, sample: Sample) {
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    /// Every sample, oldest first.
    pub fn samples(&self) -> Vec<Sample> {
        self.samples.iter().copied().collect()
    }

    /// The samples taken in the last `window` milliseconds before `now`.
    pub fn recent(&self, now: Millis, window: Millis) -> impl Iterator<Item = &Sample> {
        let from = now.saturating_sub(window);
        self.samples.iter().filter(move |s| s.at >= from)
    }

    /// How far back the history reaches from `now`, in milliseconds.
    pub fn span(&self, now: Millis) -> Millis {
        self.samples.front().map(|s| now.saturating_sub(s.at)).unwrap_or(0)
    }

    /// The average of one value over the last `window` milliseconds, if there are samples.
    pub fn average(&self, now: Millis, window: Millis, value: impl Fn(&Sample) -> f32) -> Option<f32> {
        let (sum, count) = self.recent(now, window).fold((0.0, 0), |(sum, count), s| (sum + value(s), count + 1));
        (count > 0).then(|| sum / count as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A sample with only a time and a processor load.
    fn sample(at: Millis, cpu: f32) -> Sample {
        Sample {
            at,
            cpu,
            memory: 0.0,
            swap: 0.0,
            received: 0,
            sent: 0,
            disk_read: 0,
            disk_written: 0,
            temperature: None,
        }
    }

    #[test]
    /// The ring forgets the oldest sample once it is full.
    fn drops_the_oldest_sample() {
        let mut history = History::new(2);
        history.push(sample(1, 10.0));
        history.push(sample(2, 20.0));
        history.push(sample(3, 30.0));
        let kept: Vec<Millis> = history.samples().iter().map(|s| s.at).collect();
        assert_eq!(kept, vec![2, 3]);
    }

    #[test]
    /// The average only looks at the window it is asked about.
    fn averages_over_a_window() {
        let mut history = History::new(10);
        history.push(sample(1_000, 100.0));
        history.push(sample(9_000, 40.0));
        history.push(sample(10_000, 60.0));
        assert_eq!(history.average(10_000, 2_000, |s| s.cpu), Some(50.0));
        assert_eq!(history.span(10_000), 9_000);
    }
}
