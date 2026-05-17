//! Mirror of `pm2/test/programmatic/god.mocha.js`.
//!
//! Smoke tests of the high-level God supervisor surface: start/list/stop/
//! delete + selector dispatch.

use rspm_core::types::ProcessStatus;
use rspm_daemon::god::God;
use rspm_protocol::Selector;
use rspm_tests::{fixture_app, isolated_home};

#[tokio::test]
async fn start_then_list_returns_one_online_process() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    god.start_app(fixture_app("api", "sleeper.sh"))
        .await
        .expect("start");
    let listed = god.list().await.expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].status, ProcessStatus::Online);
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn selector_by_id_targets_one_process() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    god.start_app(fixture_app("first", "sleeper.sh"))
        .await
        .expect("start");
    god.start_app(fixture_app("second", "sleeper.sh"))
        .await
        .expect("start");
    let stopped = god
        .stop_selector(&Selector::Id(0))
        .await
        .expect("stop by id");
    let zero = stopped.iter().find(|p| p.pm_id == 0).expect("id 0 present");
    let one = stopped.iter().find(|p| p.pm_id == 1).expect("id 1 present");
    assert_eq!(zero.status, ProcessStatus::Stopped);
    assert_eq!(one.status, ProcessStatus::Online);
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn delete_all_clears_the_registry_completely() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    god.start_app(fixture_app("a", "sleeper.sh"))
        .await
        .expect("start");
    god.delete_selector(&Selector::All).await.expect("delete");
    assert!(god.list().await.expect("list").is_empty());
}

#[tokio::test]
async fn stop_then_restart_round_trip() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    god.start_app(fixture_app("rt", "sleeper.sh"))
        .await
        .expect("start");
    god.stop_selector(&Selector::All).await.expect("stop");
    let restarted = god.restart_selector(&Selector::All).await.expect("restart");
    assert_eq!(restarted[0].status, ProcessStatus::Online);
    assert!(restarted[0].restart_time >= 1);
    let _ = god.delete_selector(&Selector::All).await;
}
