//! Mirror of PM2 `stop_exit_codes` behavior tested across
//! `pm2/test/programmatic/auto_restart.mocha.js` and the schema docs.
//!
//! Verifies that exit codes listed in `stop_exit_codes` are treated as an
//! intentional shutdown — the process is marked `Stopped` and the restart
//! loop short-circuits even when `auto_restart = true`.

use std::time::Duration;

use rspm_core::types::ProcessStatus;
use rspm_daemon::god::God;
use rspm_protocol::Selector;
use rspm_tests::{isolated_home, sh_app};

#[tokio::test]
async fn intentional_exit_code_does_not_restart() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let mut app = sh_app("graceful", "sleep 0.05; exit 143");
    app.auto_restart = true;
    app.stop_exit_codes = vec![143];

    god.start_app(app).await.expect("start");
    tokio::time::sleep(Duration::from_millis(400)).await;
    let listed = god.list().await.expect("list");
    assert_eq!(listed[0].status, ProcessStatus::Stopped);
    assert_eq!(listed[0].restart_time, 0);
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn non_listed_non_zero_exit_still_restarts() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let mut app = sh_app("err", "sleep 0.05; exit 7");
    app.auto_restart = true;
    app.stop_exit_codes = vec![143]; // 7 is NOT in the list

    god.start_app(app).await.expect("start");
    tokio::time::sleep(Duration::from_millis(600)).await;
    let listed = god.list().await.expect("list");
    assert!(listed[0].restart_time >= 1);
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn zero_exit_is_always_intentional() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let mut app = sh_app("ok", "sleep 0.05; exit 0");
    app.auto_restart = true;

    god.start_app(app).await.expect("start");
    tokio::time::sleep(Duration::from_millis(400)).await;
    let listed = god.list().await.expect("list");
    assert_eq!(listed[0].status, ProcessStatus::Stopped);
    assert_eq!(listed[0].restart_time, 0);
    let _ = god.delete_selector(&Selector::All).await;
}
