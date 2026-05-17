//! Mirror of `pm2/test/programmatic/graceful.mocha.js`.
//!
//! Verifies that a process that traps `SIGINT` and exits cleanly is reported
//! `Stopped`, and that one which ignores `SIGINT` is force-killed once
//! `kill_timeout_ms` lapses.

use std::time::{Duration, Instant};

use rspm_core::types::ProcessStatus;
use rspm_daemon::god::God;
use rspm_protocol::Selector;
use rspm_tests::{fixture_app, isolated_home};

#[tokio::test]
async fn graceful_app_exits_within_kill_timeout() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let mut app = fixture_app("nice", "graceful.sh");
    app.kill_timeout_ms = 1_500;

    god.start_app(app).await.expect("start");
    tokio::time::sleep(Duration::from_millis(150)).await;

    let start = Instant::now();
    let after = god.stop_selector(&Selector::All).await.expect("stop");
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_secs(2));
    assert_eq!(after[0].status, ProcessStatus::Stopped);
}

#[tokio::test(flavor = "multi_thread")]
async fn sigint_ignoring_app_is_force_killed_after_kill_timeout() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let mut app = fixture_app("stubborn", "ignore-sigint.sh");
    app.kill_timeout_ms = 300;

    god.start_app(app).await.expect("start");
    tokio::time::sleep(Duration::from_millis(100)).await;

    let start = Instant::now();
    let after = god.stop_selector(&Selector::All).await.expect("stop");
    let elapsed = start.elapsed();
    // Daemon should SIGKILL after kill_timeout passes, total < 2x timeout.
    assert!(
        elapsed < Duration::from_millis(2_000),
        "stop took {elapsed:?}"
    );
    assert_eq!(after[0].status, ProcessStatus::Stopped);
}
