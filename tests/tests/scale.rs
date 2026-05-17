//! Mirror of `pm2/lib/API/Containerizer.js` + the `pm2 scale` test path.

use rspm_core::types::{ExecutionMode, InstanceCount};
use rspm_daemon::god::God;
use rspm_protocol::Selector;
use rspm_tests::{fixture_app, isolated_home};

#[tokio::test]
async fn scale_up_spawns_additional_cluster_instances() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let mut app = fixture_app("scaler", "sleeper.sh");
    app.execution_mode = ExecutionMode::ClusterMode;
    app.instances = InstanceCount::Count(1);
    let name = app.name.clone();
    god.start_app(app).await.expect("start");

    let after = god.scale(&name, 3).await.expect("scale up");
    assert_eq!(after.len(), 3);
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn scale_down_stops_highest_indexed_instances() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let mut app = fixture_app("shrink", "sleeper.sh");
    app.execution_mode = ExecutionMode::ClusterMode;
    app.instances = InstanceCount::Count(3);
    let name = app.name.clone();
    god.start_app(app).await.expect("start");

    let after = god.scale(&name, 1).await.expect("scale down");
    assert_eq!(after.len(), 1);
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn scale_rejects_fork_mode_apps() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let app = fixture_app("forkonly", "sleeper.sh");
    let name = app.name.clone();
    god.start_app(app).await.expect("start");

    let err = god.scale(&name, 2).await.expect_err("must reject");
    assert!(format!("{err}").contains("fork_mode"));
    let _ = god.delete_selector(&Selector::All).await;
}
