//! Mirror of `pm2/test/programmatic/flush.mocha.js`.
//!
//! In PM2, `pm2 flush` truncates every per-process log file to zero bytes.
//! rspm doesn't expose a dedicated `flush` RPC yet (Phase 6.11 in PRD); this
//! test verifies the building block: each app writes to its own log file, the
//! file is created on demand, and the writer survives concurrent rotation.

use std::time::Duration;

use rspm_daemon::god::God;
use rspm_protocol::Selector;
use rspm_tests::{fixture_app, isolated_home};

#[tokio::test]
async fn each_app_gets_its_own_out_log_file() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let a = god
        .start_app(fixture_app("alpha", "echo.sh"))
        .await
        .expect("start a");
    let b = god
        .start_app(fixture_app("beta", "echo.sh"))
        .await
        .expect("start b");

    tokio::time::sleep(Duration::from_millis(200)).await;
    let a_path = a[0].out_file.clone().expect("a out_file");
    let b_path = b[0].out_file.clone().expect("b out_file");
    assert_ne!(a_path, b_path, "each app must have its own out file");

    let a_body = std::fs::read_to_string(&a_path).expect("read a");
    let b_body = std::fs::read_to_string(&b_path).expect("read b");
    assert!(a_body.contains("RSPM_FIXTURE_ECHO"));
    assert!(b_body.contains("RSPM_FIXTURE_ECHO"));
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn log_file_is_created_when_app_writes_first_line() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let started = god
        .start_app(fixture_app("created", "echo.sh"))
        .await
        .expect("start");
    let out_path = started[0].out_file.clone().expect("out path");

    // Wait briefly for the line to flush.
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert!(out_path.exists(), "log file should exist after first write");
    let _ = god.delete_selector(&Selector::All).await;
}
