//! Shared helpers for the rspm parity test suite.
//!
//! Each `tests/*.rs` file mirrors a file from `pm2/test/programmatic/` so
//! parity gaps are easy to spot by cross-referencing the original Mocha
//! source. Keep helper surface minimal — tests should read top-to-bottom
//! without jumping into this module too often.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use rspm_core::paths::RspmHome;
use rspm_core::types::AppConfig;

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Returns the path to `tests/fixtures/<rel>` so tests can reference shell
/// scripts checked into the repository instead of inlining them.
pub fn fixture(rel: &str) -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("fixtures").join(rel)
}

/// Allocates an isolated `$RSPM_HOME` per test so tests can run in parallel
/// without colliding on `rpc.sock`, `dump.rspm`, or per-app log files.
pub fn isolated_home() -> (tempfile::TempDir, RspmHome) {
    let dir = tempfile::tempdir().expect("create temp dir");
    let home = RspmHome::new(dir.path());
    home.ensure().expect("ensure home dirs");
    (dir, home)
}

/// Builds an [`AppConfig`] that runs the given shell command (`-c "<cmd>"`).
/// Each call also bumps a monotonic counter so the derived name is unique
/// across the suite.
pub fn sh_app(name: &str, cmd: &str) -> AppConfig {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut app = AppConfig::from_script(PathBuf::from("/bin/sh"), Some(format!("{name}-{n}")));
    app.args = vec!["-c".into(), cmd.to_owned()];
    app.auto_restart = false;
    app.kill_timeout_ms = 500;
    app
}

/// Builds an app that runs one of the bundled fixtures from
/// `tests/fixtures/`.
pub fn fixture_app(name: &str, fixture_rel: &str) -> AppConfig {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut app = AppConfig::from_script(fixture(fixture_rel), Some(format!("{name}-{n}")));
    app.interpreter = Some(PathBuf::from("/bin/sh"));
    app.auto_restart = false;
    app.kill_timeout_ms = 500;
    app
}
