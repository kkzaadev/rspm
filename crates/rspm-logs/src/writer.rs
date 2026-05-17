//! Append-only log writer with optional timestamp prefix and merge_logs
//! labeling.
//!
//! Models the line-by-line write path PM2 uses inside `God.bus.on('data')`:
//! - When `prefix_timestamp` is set, every line is prefixed with the formatted
//!   timestamp (matching `log_date_format`, falling back to
//!   [`crate::timestamp::DEFAULT_LOG_DATE_FORMAT`]).
//! - When `merge_label` is set, the daemon-supplied label (typically the app
//!   name without instance suffix) is wrapped in `[label]` and prefixed so
//!   multi-instance apps land in a single shared log without an instance id
//!   in the line.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use rspm_core::Result;

use crate::rotator::Rotator;
use crate::timestamp::{DEFAULT_LOG_DATE_FORMAT, prefix_with};

/// Log writer options.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LogOpts {
    /// Prefix each line with a timestamp.
    pub prefix_timestamp: bool,
    /// Optional strftime pattern. Falls back to
    /// [`crate::timestamp::DEFAULT_LOG_DATE_FORMAT`] when
    /// `prefix_timestamp` is true and this field is `None` or empty.
    pub log_date_format: Option<String>,
    /// Optional label inserted between the timestamp and the line content.
    /// PM2 uses the app name here when `merge_logs` is true.
    pub merge_label: Option<String>,
    /// Maximum bytes per active file before rotation; `None` disables it.
    pub max_bytes: Option<u64>,
    /// Maximum number of archives kept when rotating. `0` clamps to `1`.
    pub max_archives: usize,
}

/// Append-only log writer.
#[derive(Debug)]
pub struct LogWriter {
    path: PathBuf,
    file: File,
    opts: LogOpts,
    rotator: Option<Rotator>,
}

impl LogWriter {
    /// Opens a log writer in append mode.
    ///
    /// ```
    /// # fn demo(path: &std::path::Path) -> rspm_core::Result<()> {
    /// let _writer = rspm_logs::LogWriter::new(path, rspm_logs::LogOpts::default())?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(path: &Path, opts: LogOpts) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new().create(true).append(true).open(path)?;
        let rotator = opts.max_bytes.map(|limit| {
            Rotator::new(path.to_path_buf(), Some(limit))
                .with_max_archives(opts.max_archives.max(1))
        });
        Ok(Self {
            path: path.to_path_buf(),
            file,
            opts,
            rotator,
        })
    }

    /// Returns the writer path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Writes one already-formatted line. A trailing `\n` is appended so
    /// callers can pass line content without terminator.
    pub fn write_line(&mut self, line: &[u8]) -> Result<()> {
        // Rotate before writing so the active file contains the line that
        // just triggered the rollover, not the one before it.
        self.maybe_rotate()?;
        let text = String::from_utf8_lossy(line);
        let prefixed = self.format_line(&text);
        self.file.write_all(prefixed.as_bytes())?;
        self.file.write_all(b"\n")?;
        self.file.flush()?;
        Ok(())
    }

    fn maybe_rotate(&mut self) -> Result<()> {
        let Some(rotator) = self.rotator.as_mut() else {
            return Ok(());
        };
        let rotated = rotator.rotate_if_needed()?;
        if rotated {
            self.file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)?;
        }
        Ok(())
    }

    fn format_line(&self, line: &str) -> String {
        let mut out = String::new();
        if self.opts.prefix_timestamp {
            let pattern = self
                .opts
                .log_date_format
                .as_deref()
                .filter(|p| !p.is_empty())
                .unwrap_or(DEFAULT_LOG_DATE_FORMAT);
            out.push_str(&prefix_with(pattern, ""));
        }
        if let Some(label) = self
            .opts
            .merge_label
            .as_deref()
            .filter(|label| !label.is_empty())
        {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push('[');
            out.push_str(label);
            out.push(']');
            out.push(' ');
        }
        if out.is_empty() {
            line.to_owned()
        } else {
            format!("{out}{line}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn writes_plain_line_with_newline() {
        let dir = tempdir().expect("temp");
        let path = dir.path().join("plain.log");
        let mut w = LogWriter::new(&path, LogOpts::default()).expect("writer");
        w.write_line(b"hello").expect("write");
        let body = std::fs::read_to_string(&path).expect("read");
        assert_eq!(body, "hello\n");
    }

    #[test]
    fn writes_timestamp_prefix_when_enabled() {
        let dir = tempdir().expect("temp");
        let path = dir.path().join("ts.log");
        let opts = LogOpts {
            prefix_timestamp: true,
            log_date_format: Some("%Y".to_owned()),
            ..LogOpts::default()
        };
        let mut w = LogWriter::new(&path, opts).expect("writer");
        w.write_line(b"hello").expect("write");
        let body = std::fs::read_to_string(&path).expect("read");
        assert!(body.ends_with("hello\n"));
        assert!(body.len() > "hello\n".len(), "timestamp prefix missing");
    }

    #[test]
    fn writes_merge_label_when_enabled() {
        let dir = tempdir().expect("temp");
        let path = dir.path().join("merge.log");
        let opts = LogOpts {
            merge_label: Some("api".to_owned()),
            ..LogOpts::default()
        };
        let mut w = LogWriter::new(&path, opts).expect("writer");
        w.write_line(b"hello").expect("write");
        let body = std::fs::read_to_string(&path).expect("read");
        assert_eq!(body, "[api] hello\n");
    }

    #[test]
    fn rotates_when_max_bytes_exceeded() {
        let dir = tempdir().expect("temp");
        let path = dir.path().join("rot.log");
        let opts = LogOpts {
            max_bytes: Some(10),
            max_archives: 2,
            ..LogOpts::default()
        };
        let mut w = LogWriter::new(&path, opts).expect("writer");
        w.write_line(b"line-aaaa").expect("write1");
        w.write_line(b"line-bbbb").expect("write2");

        let body = std::fs::read_to_string(&path).expect("read current");
        assert_eq!(body, "line-bbbb\n");
        let archive = path.with_file_name("rot.1.log");
        let archive_body = std::fs::read_to_string(&archive).expect("read archive");
        assert_eq!(archive_body, "line-aaaa\n");
    }
}
