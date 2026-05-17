//! Mirror of `pm2/test/programmatic/auto_restart.mocha.js`.
//!
//! Verifies that crashed apps auto-restart in fork mode and cluster mode, and
//! that `auto_restart = false` disables the loop.

use std::time::Duration;

use rspm_core::types::InstanceCount;
use rspm_daemon::god::God;
use rspm_protocol::Selector;
use rspm_tests::{fixture_app, isolated_home};

#[tokio::test]
async fn should_start_a_failing_app_in_fork_mode() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let mut app = fixture_app("throw", "throw.sh");
    app.auto_restart = true;
    app.max_restarts = 2;

    god.start_app(app).await.expect("start");
    // give the child time to crash + auto-restart at least once.
    tokio::time::sleep(Duration::from_millis(800)).await;

    let listed = god.list().await.expect("list");
    assert!(listed[0].restart_time >= 1, "expected restart_time >= 1");
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn should_start_a_failing_app_in_cluster_mode() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let mut app = fixture_app("throw_cluster", "throw.sh");
    app.auto_restart = true;
    app.instances = InstanceCount::Count(2);
    app.max_restarts = 2;

    god.start_app(app).await.expect("start");
    tokio::time::sleep(Duration::from_millis(800)).await;

    let listed = god.list().await.expect("list");
    assert_eq!(listed.len(), 2);
    assert!(listed.iter().any(|p| p.restart_time >= 1));
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn auto_restart_false_does_not_restart_on_crash() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let mut app = fixture_app("no_restart", "throw.sh");
    app.auto_restart = false;

    god.start_app(app).await.expect("start");
    tokio::time::sleep(Duration::from_millis(500)).await;

    let listed = god.list().await.expect("list");
    assert_eq!(listed[0].restart_time, 0);
    let _ = god.delete_selector(&Selector::All).await;
}
