//! Mirror of `pm2/test/programmatic/reload-locker.mocha.js`.
//!
//! Concurrent reload requests must not race or double-spawn replacements.
//! rspm serializes all RPC calls behind a single `tokio::Mutex<God>` so back
//! to back reloads on the same app must succeed and leave exactly one
//! process in the registry.

use std::time::Duration;

use rspm_core::types::{ExecutionMode, InstanceCount, ProcessStatus};
use rspm_daemon::god::God;
use rspm_protocol::Selector;
use rspm_tests::{fixture_app, isolated_home};

#[tokio::test]
async fn back_to_back_reloads_leave_one_running_process() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let mut app = fixture_app("lock", "sleeper.sh");
    app.execution_mode = ExecutionMode::ClusterMode;
    app.instances = InstanceCount::Count(1);
    god.start_app(app).await.expect("start");

    god.reload_selector(&Selector::All).await.expect("reload 1");
    god.reload_selector(&Selector::All).await.expect("reload 2");
    god.reload_selector(&Selector::All).await.expect("reload 3");
    tokio::time::sleep(Duration::from_millis(150)).await;

    let listed = god.list().await.expect("list");
    assert_eq!(listed.len(), 1, "exactly one process should remain");
    assert_eq!(listed[0].status, ProcessStatus::Online);
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn reload_after_stop_still_completes() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let mut app = fixture_app("postsop", "sleeper.sh");
    app.execution_mode = ExecutionMode::ClusterMode;
    app.instances = InstanceCount::Count(1);
    god.start_app(app).await.expect("start");
    god.stop_selector(&Selector::All).await.expect("stop");
    // After a stop the cluster replacement loop must still find the process
    // (just in stopped state) and bring it back online.
    let after = god
        .reload_selector(&Selector::All)
        .await
        .expect("reload after stop");
    assert_eq!(after[0].status, ProcessStatus::Online);
    let _ = god.delete_selector(&Selector::All).await;
}
