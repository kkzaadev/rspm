//! Mirror of `pm2/test/programmatic/dump.mocha.js`.
//!
//! Checks save -> tear-down -> resurrect round-trip preserves the app list.

use rspm_core::types::ProcessStatus;
use rspm_daemon::god::God;
use rspm_protocol::Selector;
use rspm_tests::{fixture_app, isolated_home};

#[tokio::test]
async fn save_writes_dump_file_with_one_entry_per_app() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home.clone());
    god.start_app(fixture_app("a", "sleeper.sh"))
        .await
        .expect("start a");
    god.start_app(fixture_app("b", "sleeper.sh"))
        .await
        .expect("start b");

    let saved = god.save().await.expect("save");
    assert_eq!(saved, 2);
    let body = std::fs::read_to_string(home.dump_file()).expect("read dump");
    let entries: Vec<serde_json::Value> = serde_json::from_str(&body).expect("dump json");
    assert_eq!(entries.len(), 2);

    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn resurrect_restores_apps_after_daemon_teardown() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home.clone());
    god.start_app(fixture_app("survivor", "sleeper.sh"))
        .await
        .expect("start");
    let saved = god.save().await.expect("save");
    assert_eq!(saved, 1);

    god.delete_selector(&Selector::All).await.expect("teardown");
    drop(god);

    let mut fresh = God::new(home);
    let started = fresh.resurrect().await.expect("resurrect");
    assert_eq!(started.len(), 1);
    assert_eq!(started[0].status, ProcessStatus::Online);
    let _ = fresh.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn resurrect_with_no_dump_file_returns_empty_list() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let started = god.resurrect().await.expect("resurrect empty");
    assert!(started.is_empty());
}
