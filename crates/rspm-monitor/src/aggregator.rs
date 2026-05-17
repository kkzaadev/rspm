//! Rolling-window aggregator for per-PID CPU and memory samples.
//!
//! `pm2` smooths metrics over the worker interval. We do the same by keeping
//! the last `N` samples per PID and exposing the arithmetic mean. The buffer
//! is intentionally small (default 10) so memory stays predictable as the
//! supervised process set grows.

use std::collections::HashMap;
use std::collections::VecDeque;

/// Default sample window length, matching PM2's smoothing behavior closely.
pub const DEFAULT_WINDOW: usize = 10;

/// Returns a CPU value rounded to one decimal place.
///
/// ```
/// assert_eq!(rspm_monitor::aggregator::round_cpu(12.349), 12.3);
/// ```
pub fn round_cpu(value: f32) -> f32 {
    (value * 10.0).round() / 10.0
}

/// Sliding window averaging buffer keyed by OS PID.
#[derive(Debug, Default)]
pub struct Aggregator {
    window: usize,
    per_pid: HashMap<u32, VecDeque<Sample>>,
}

#[derive(Copy, Clone, Debug)]
struct Sample {
    cpu: f32,
    mem: u64,
}

impl Aggregator {
    /// Creates a new aggregator with the given window length.
    ///
    /// A window of `0` is treated as `1`; otherwise the latest `window`
    /// samples per pid are retained.
    ///
    /// ```
    /// let agg = rspm_monitor::aggregator::Aggregator::new(5);
    /// assert_eq!(agg.window(), 5);
    /// ```
    pub fn new(window: usize) -> Self {
        Self {
            window: window.max(1),
            per_pid: HashMap::new(),
        }
    }

    /// Returns the configured window length.
    pub fn window(&self) -> usize {
        self.window
    }

    /// Records a new sample for a pid.
    pub fn record(&mut self, pid: u32, cpu_percent: f32, mem_bytes: u64) {
        let buf = self.per_pid.entry(pid).or_default();
        if buf.len() == self.window {
            buf.pop_front();
        }
        buf.push_back(Sample {
            cpu: cpu_percent,
            mem: mem_bytes,
        });
    }

    /// Returns the average `(cpu_percent, mem_bytes)` over the current window.
    pub fn average(&self, pid: u32) -> Option<(f32, u64)> {
        let buf = self.per_pid.get(&pid)?;
        if buf.is_empty() {
            return None;
        }
        let len = buf.len() as f32;
        let cpu = round_cpu(buf.iter().map(|s| s.cpu).sum::<f32>() / len);
        let mem_total: u128 = buf.iter().map(|s| u128::from(s.mem)).sum();
        let mem = u64::try_from(mem_total / buf.len() as u128).unwrap_or(u64::MAX);
        Some((cpu, mem))
    }

    /// Drops all samples for a pid (call when a process is removed).
    pub fn forget(&mut self, pid: u32) {
        self.per_pid.remove(&pid);
    }

    /// Number of pids currently tracked.
    pub fn tracked_pids(&self) -> usize {
        self.per_pid.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn averages_within_window() {
        let mut agg = Aggregator::new(3);
        agg.record(42, 10.0, 100);
        agg.record(42, 30.0, 200);
        agg.record(42, 20.0, 300);
        let (cpu, mem) = agg.average(42).expect("present");
        assert_eq!(cpu, 20.0);
        assert_eq!(mem, 200);
    }

    #[test]
    fn slides_window() {
        let mut agg = Aggregator::new(2);
        agg.record(1, 10.0, 100);
        agg.record(1, 10.0, 100);
        agg.record(1, 40.0, 400); // pops the oldest
        let (cpu, mem) = agg.average(1).expect("present");
        assert_eq!(cpu, 25.0);
        assert_eq!(mem, 250);
    }

    #[test]
    fn forget_removes_history() {
        let mut agg = Aggregator::new(5);
        agg.record(7, 50.0, 1024);
        assert_eq!(agg.tracked_pids(), 1);
        agg.forget(7);
        assert_eq!(agg.tracked_pids(), 0);
        assert!(agg.average(7).is_none());
    }

    #[test]
    fn zero_window_clamps_to_one() {
        let agg = Aggregator::new(0);
        assert_eq!(agg.window(), 1);
    }
}
