//! Monitoring sample types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// CPU sample for a process.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CpuSample {
    /// Operating system PID.
    pub pid: u32,
    /// CPU percentage.
    pub percent: f32,
    /// Sampling time.
    pub sampled_at: DateTime<Utc>,
}

/// Memory sample for a process.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemSample {
    /// Operating system PID.
    pub pid: u32,
    /// Resident memory in bytes.
    pub bytes: u64,
    /// Sampling time.
    pub sampled_at: DateTime<Utc>,
}
