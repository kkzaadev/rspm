//! Mirror of `pm2/test/programmatic/cluster.mocha.js`.
//!
//! In rspm cluster-mode is implemented via `SO_REUSEPORT` env hints — there
//! is no Node-cluster IPC channel. We test the visible contract: env vars
//! reach the child, multiple instances are spawned with distinct
//! `instance_index`, and soft reload swaps in a fresh worker.

use std::time::Duration;

use rspm_core::types::{ExecutionMode, InstanceCount, ProcessStatus};
use rspm_daemon::god::God;
use rspm_protocol::Selector;
use rspm_tests::{fixture_app, isolated_home};

#[tokio::test]
async fn cluster_mode_propagates_cluster_env_to_child() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let mut app = fixture_app("cluster", "env-print.sh");
    app.execution_mode = ExecutionMode::ClusterMode;
    app.instances = InstanceCount::Count(2);
    god.start_app(app).await.expect("start");

    tokio::time::sleep(Duration::from_millis(400)).await;
    let listed = god.list().await.expect("list");
    assert_eq!(listed.len(), 2);

    let out_path = listed[0].out_file.clone().expect("out_file");
    let body = std::fs::read_to_string(&out_path).expect("read out");
    assert!(body.contains("RSPM_CLUSTER=1"), "missing in: {body}");
    assert!(
        body.contains("RSPM_EXEC_MODE=cluster_mode"),
        "missing in: {body}"
    );
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn reload_on_cluster_mode_performs_rolling_swap() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let mut app = fixture_app("reload", "sleeper.sh");
    app.execution_mode = ExecutionMode::ClusterMode;
    app.instances = InstanceCount::Count(1);
    god.start_app(app).await.expect("start");

    let before = god.list().await.expect("list before");
    let before_id = before[0].pm_id;

    let after = god.reload_selector(&Selector::All).await.expect("reload");
    let after_id = after[0].pm_id;
    assert_ne!(
        before_id, after_id,
        "cluster reload must allocate a fresh pm_id"
    );
    assert_eq!(after[0].status, ProcessStatus::Online);
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn reload_on_fork_mode_falls_back_to_restart_same_id() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let app = fixture_app("forky", "sleeper.sh");
    let started = god.start_app(app).await.expect("start");
    let before_id = started[0].pm_id;

    let after = god.reload_selector(&Selector::All).await.expect("reload");
    assert_eq!(
        after[0].pm_id, before_id,
        "fork-mode reload keeps the same pm_id (acts like restart)"
    );
    let _ = god.delete_selector(&Selector::All).await;
}
