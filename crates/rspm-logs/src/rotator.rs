//! Size-based log rotator.
//!
//! Each call to [`Rotator::rotate_if_needed`] inspects the configured log
//! file size and, when it exceeds the configured limit, shifts archives:
//!
//! ```text
//! app-out.log    -> app-out.1.log
//! app-out.1.log  -> app-out.2.log
//! ...
//! app-out.N.log  -> dropped when N == max_archives
//! ```
//!
//! The active path is left in a removed state so the writer can re-open a
//! fresh, empty file. PM2 delegates this to the `pm2-logrotate` module; we
//! ship the equivalent inside the daemon so basic rotation works out of the
//! box without an installed module.

use std::path::{Path, PathBuf};

use rspm_core::Result;

/// Default number of archives kept when rotation is enabled without an
/// explicit limit.
pub const DEFAULT_MAX_ARCHIVES: usize = 5;

/// Size-based log rotator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rotator {
    path: PathBuf,
    max_bytes: Option<u64>,
    max_archives: usize,
}

impl Rotator {
    /// Creates a rotator for a log path.
    ///
    /// ```
    /// let rotator = rspm_logs::rotator::Rotator::new("/tmp/app.log", Some(1024));
    /// assert_eq!(rotator.max_bytes(), Some(1024));
    /// ```
    pub fn new(path: impl Into<PathBuf>, max_bytes: Option<u64>) -> Self {
        Self {
            path: path.into(),
            max_bytes,
            max_archives: DEFAULT_MAX_ARCHIVES,
        }
    }

    /// Sets the maximum number of archive files retained.
    pub fn with_max_archives(mut self, max_archives: usize) -> Self {
        self.max_archives = max_archives.max(1);
        self
    }

    /// Returns the configured byte limit.
    pub fn max_bytes(&self) -> Option<u64> {
        self.max_bytes
    }

    /// Returns the configured archive retention limit.
    pub fn max_archives(&self) -> usize {
        self.max_archives
    }

    /// Rotates the active log when its on-disk size exceeds the configured
    /// `max_bytes`. Returns `Ok(true)` when rotation happened, `Ok(false)`
    /// otherwise.
    pub fn rotate_if_needed(&mut self) -> Result<bool> {
        let Some(limit) = self.max_bytes else {
            return Ok(false);
        };
        let size = match std::fs::metadata(&self.path) {
            Ok(meta) => meta.len(),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(err) => return Err(err.into()),
        };
        if size < limit {
            return Ok(false);
        }

        // Drop the oldest archive that would fall off the retention window.
        let drop_target = archive_path(&self.path, self.max_archives);
        if drop_target.exists() {
            std::fs::remove_file(&drop_target)?;
        }

        // Shift remaining archives: .N -> .(N+1) down to 1.
        for idx in (1..self.max_archives).rev() {
            let from = archive_path(&self.path, idx);
            let to = archive_path(&self.path, idx + 1);
            if from.exists() {
                std::fs::rename(&from, &to)?;
            }
        }

        // Active file becomes .1.
        let first = archive_path(&self.path, 1);
        std::fs::rename(&self.path, &first)?;
        Ok(true)
    }
}

/// Computes the archive path `base.<idx>.<ext>`.
pub fn archive_path(base: &Path, idx: usize) -> PathBuf {
    let dir = base.parent().unwrap_or_else(|| Path::new("."));
    let stem = base
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    match base.extension().and_then(|s| s.to_str()) {
        Some(ext) if !ext.is_empty() => dir.join(format!("{stem}.{idx}.{ext}")),
        _ => dir.join(format!("{stem}.{idx}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn rotates_on_overflow() {
        let dir = tempdir().expect("temp");
        let path = dir.path().join("app.log");
        std::fs::write(&path, b"AAAAAAAAAAAAAAAA").expect("write");

        let mut rotator = Rotator::new(path.clone(), Some(8)).with_max_archives(2);
        assert!(rotator.rotate_if_needed().expect("rotate"));
        assert!(!path.exists(), "active file should have been rotated away");
        let archive = path.with_file_name("app.1.log");
        assert!(archive.exists());

        // Second rotation should shift .1 -> .2 and need a new active file.
        std::fs::write(&path, b"BBBBBBBBBBBBBBBB").expect("write2");
        assert!(rotator.rotate_if_needed().expect("rotate2"));
        assert!(path.with_file_name("app.2.log").exists());
        let first = std::fs::read_to_string(path.with_file_name("app.1.log")).expect("read");
        assert!(first.starts_with("BBBB"));
    }

    #[test]
    fn noop_when_under_limit() {
        let dir = tempdir().expect("temp");
        let path = dir.path().join("under.log");
        std::fs::write(&path, b"hi").expect("write");
        let mut rotator = Rotator::new(path.clone(), Some(64));
        assert!(!rotator.rotate_if_needed().expect("rotate"));
        assert!(path.exists());
    }

    #[test]
    fn noop_when_no_limit() {
        let dir = tempdir().expect("temp");
        let path = dir.path().join("noop.log");
        std::fs::write(&path, b"hi").expect("write");
        let mut rotator = Rotator::new(path, None);
        assert!(!rotator.rotate_if_needed().expect("rotate"));
    }

    #[test]
    fn archive_path_handles_no_extension() {
        let path = PathBuf::from("/tmp/app");
        let arch = archive_path(&path, 3);
        assert_eq!(arch, PathBuf::from("/tmp/app.3"));
    }
}
