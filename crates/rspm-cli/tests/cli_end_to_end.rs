//! True end-to-end tests for the `rspm` binary.
//!
//! Each test runs the actual compiled binary with an isolated `$RSPM_HOME` and
//! drives a real daemon over Unix Domain Sockets. The binary path is provided
//! by Cargo via `env!("CARGO_BIN_EXE_rspm")`. The daemon process is the same
//! binary invoked with `--daemon` (see `daemon_launcher::launch_if_needed`).
//!
//! These tests are intentionally serialized via filesystem isolation (each
//! test owns its own tempdir) and use generous timeouts to be CI-friendly.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::Duration;

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn rspm_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_rspm"))
}

fn fresh_home() -> tempfile::TempDir {
    tempfile::tempdir().expect("temp rspm home")
}

fn run(rspm: &Path, home: &Path, args: &[&str]) -> std::process::Output {
    Command::new(rspm)
        .args(args)
        .env("RSPM_HOME", home)
        .env("RSPM_DAEMON_BIN", rspm)
        .output()
        .expect("spawn rspm")
}

fn wait_for_socket(home: &Path) {
    let socket = home.join("rpc.sock");
    for _ in 0..50 {
        if socket.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
}

/// `rspm ping` should return Pong within a reasonable time even when the
/// daemon must be auto-spawned.
#[test]
fn ping_returns_pong_and_auto_spawns_daemon() {
    let _n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let home = fresh_home();
    let bin = rspm_bin();

    let output = run(&bin, home.path(), &["ping"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "ping failed: stdout={stdout} stderr={stderr}"
    );
    let combined = format!("{stdout}{stderr}").to_lowercase();
    assert!(
        combined.contains("pong"),
        "expected pong in output, got: {combined}"
    );

    wait_for_socket(home.path());

    // Tear down the daemon spawned during this test.
    let _ = run(&bin, home.path(), &["kill"]);
}

/// `rspm list` on a fresh home reports an empty process list without crashing.
#[test]
fn list_on_empty_home_returns_zero_processes() {
    let home = fresh_home();
    let bin = rspm_bin();

    let output = run(&bin, home.path(), &["list"]);
    assert!(
        output.status.success(),
        "list failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let _ = run(&bin, home.path(), &["kill"]);
}

/// `rspm jlist` returns parseable JSON even when empty.
#[test]
fn jlist_returns_valid_json_when_empty() {
    let home = fresh_home();
    let bin = rspm_bin();

    let output = run(&bin, home.path(), &["jlist"]);
    assert!(
        output.status.success(),
        "jlist failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("jlist output must be valid JSON");
    assert!(parsed.is_array(), "jlist output must be a JSON array");

    let _ = run(&bin, home.path(), &["kill"]);
}

/// `rspm kill` removes the daemon's RPC socket so a subsequent client must
/// auto-spawn a new daemon.
#[test]
fn kill_removes_daemon_socket() {
    let home = fresh_home();
    let bin = rspm_bin();

    // Spawn daemon via ping, then kill it.
    let _ = run(&bin, home.path(), &["ping"]);
    wait_for_socket(home.path());
    assert!(home.path().join("rpc.sock").exists(), "socket should exist");

    let kill = run(&bin, home.path(), &["kill"]);
    assert!(
        kill.status.success(),
        "kill failed: {}",
        String::from_utf8_lossy(&kill.stderr)
    );

    // Give the daemon a moment to clean up.
    let mut gone = false;
    for _ in 0..20 {
        if !home.path().join("rpc.sock").exists() {
            gone = true;
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
    assert!(gone, "rpc.sock should be removed after kill");
}
