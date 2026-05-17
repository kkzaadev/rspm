//! Per-PID CPU and memory sampler backed by [`sysinfo`].
//!
//! The sampler keeps a `sysinfo::System` instance alive across calls so that
//! [`sysinfo`] can compute the CPU usage delta between two refreshes. The
//! intent matches `pm2/lib/God/SystemData.js`, which polls `/proc/<pid>/stat`
//! at a fixed cadence and reports cpu percent and resident memory.

use std::time::Duration;

use chrono::Utc;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

use rspm_core::types::{CpuSample, MemSample};

use crate::aggregator::round_cpu;

/// CPU and memory sampler.
#[derive(Debug)]
pub struct Sampler {
    interval: Duration,
    system: System,
}

impl Sampler {
    /// Creates a sampler with a fixed sampling cadence.
    ///
    /// ```
    /// use std::time::Duration;
    /// let _ = rspm_monitor::Sampler::new(Duration::from_secs(1));
    /// ```
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            system: System::new(),
        }
    }

    /// Samples cpu percent and resident memory for the given PIDs.
    ///
    /// Returns a tuple per PID. PIDs that are not visible (already exited or
    /// permission denied) report zeros.
    pub fn sample(&mut self, pids: &[u32]) -> Vec<(u32, CpuSample, MemSample)> {
        if pids.is_empty() {
            return Vec::new();
        }

        let pid_list: Vec<Pid> = pids.iter().copied().map(Pid::from_u32).collect();
        let refresh = ProcessRefreshKind::new().with_cpu().with_memory();
        self.system
            .refresh_processes_specifics(ProcessesToUpdate::Some(&pid_list), true, refresh);

        let now = Utc::now();
        pids.iter()
            .copied()
            .map(|pid| {
                let (percent, bytes) = match self.system.process(Pid::from_u32(pid)) {
                    Some(proc) => (round_cpu(proc.cpu_usage()), proc.memory()),
                    None => (0.0, 0),
                };
                (
                    pid,
                    CpuSample {
                        pid,
                        percent,
                        sampled_at: now,
                    },
                    MemSample {
                        pid,
                        bytes,
                        sampled_at: now,
                    },
                )
            })
            .collect()
    }

    /// Returns the sampling interval.
    ///
    /// ```
    /// use std::time::Duration;
    /// assert_eq!(
    ///     rspm_monitor::Sampler::new(Duration::from_secs(2)).interval(),
    ///     Duration::from_secs(2),
    /// );
    /// ```
    pub fn interval(&self) -> Duration {
        self.interval
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_self_pid_reports_nonzero_memory() {
        let mut sampler = Sampler::new(Duration::from_millis(100));
        let pid = std::process::id();
        // First call seeds the cpu delta; second call should yield a meaningful sample.
        let _ = sampler.sample(&[pid]);
        std::thread::sleep(Duration::from_millis(60));
        let result = sampler.sample(&[pid]);
        assert_eq!(result.len(), 1);
        let (_, _, mem) = &result[0];
        assert!(mem.bytes > 0, "expected memory > 0 for self pid");
    }

    #[test]
    fn unknown_pid_yields_zero() {
        let mut sampler = Sampler::new(Duration::from_millis(100));
        let result = sampler.sample(&[u32::MAX - 1]);
        assert_eq!(result.len(), 1);
        let (_, cpu, mem) = &result[0];
        assert_eq!(cpu.percent, 0.0);
        assert_eq!(mem.bytes, 0);
    }

    #[test]
    fn empty_pids_returns_empty() {
        let mut sampler = Sampler::new(Duration::from_millis(100));
        assert!(sampler.sample(&[]).is_empty());
    }
}
