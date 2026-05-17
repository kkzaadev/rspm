//! End-to-end tests modeling the PM2 parity matrix from PRD §9.1.
//!
//! Each test runs against a fresh `God` supervisor backed by a tempdir
//! `RSPM_HOME`, exercising the same code paths the daemon binary uses in
//! production. Real subprocesses are spawned via `/bin/sh` so the tests are
//! Unix-only.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use rspm_core::paths::RspmHome;
use rspm_core::types::{AppConfig, InstanceCount, ProcessStatus};
use rspm_daemon::god::God;
use rspm_protocol::Selector;

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn make_home() -> (tempfile::TempDir, RspmHome) {
    let dir = tempfile::tempdir().expect("temp dir");
    let home = RspmHome::new(dir.path());
    home.ensure().expect("ensure home");
    (dir, home)
}

fn long_running_app(name: &str) -> AppConfig {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut app = AppConfig::from_script(PathBuf::from("/bin/sh"), Some(format!("{name}-{n}")));
    app.args = vec![
        "-c".into(),
        "while true; do echo tick; sleep 0.1; done".into(),
    ];
    app.auto_restart = false;
    app.kill_timeout_ms = 500;
    app
}

/// PRD §9.1 row: `start app.js` → process online.
#[tokio::test]
async fn start_app_marks_process_online() {
    let (_dir, home) = make_home();
    let mut god = God::new(home);
    let started = god.start_app(long_running_app("api")).await.expect("start");
    assert_eq!(started.len(), 1);
    assert_eq!(started[0].status, ProcessStatus::Online);
    let _ = god.stop_selector(&Selector::All).await;
}

/// PRD §9.1 row: `start --name X` → name preserved.
#[tokio::test]
async fn start_preserves_user_provided_name() {
    let (_dir, home) = make_home();
    let mut god = God::new(home);
    let mut app = long_running_app("base");
    app.name = "custom-name".into();
    let started = god.start_app(app).await.expect("start");
    assert_eq!(started[0].name, "custom-name");
    let _ = god.stop_selector(&Selector::All).await;
}

/// PRD §9.1 row: `restart 0` → restart_time bertambah.
#[tokio::test]
async fn restart_increments_restart_time() {
    let (_dir, home) = make_home();
    let mut god = God::new(home);
    god.start_app(long_running_app("restart-counter"))
        .await
        .expect("start");

    let after = god
        .restart_selector(&Selector::Id(0))
        .await
        .expect("restart");
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].restart_time, 1);
    assert_eq!(after[0].status, ProcessStatus::Online);
    let _ = god.stop_selector(&Selector::All).await;
}

/// PRD §9.1 row: `stop all` → semua stopped.
#[tokio::test]
async fn stop_all_marks_every_process_stopped() {
    let (_dir, home) = make_home();
    let mut god = God::new(home);
    god.start_app(long_running_app("a")).await.expect("start a");
    god.start_app(long_running_app("b")).await.expect("start b");

    let listed = god.stop_selector(&Selector::All).await.expect("stop all");
    assert_eq!(listed.len(), 2);
    for info in listed {
        assert_eq!(info.status, ProcessStatus::Stopped);
    }
}

/// PRD §9.1 row: `delete all` → list kosong.
#[tokio::test]
async fn delete_all_clears_registry() {
    let (_dir, home) = make_home();
    let mut god = God::new(home);
    god.start_app(long_running_app("a")).await.expect("start");
    god.delete_selector(&Selector::All).await.expect("delete");
    assert!(god.list().await.expect("list").is_empty());
}

/// PRD §9.1 row: `save && kill && resurrect` → state restored.
#[tokio::test]
async fn save_then_resurrect_round_trip() {
    let (_dir, home) = make_home();
    let mut god = God::new(home.clone());
    god.start_app(long_running_app("survivor"))
        .await
        .expect("start");

    let saved = god.save().await.expect("save");
    assert_eq!(saved, 1);

    // Simulate daemon restart: tear everything down, then create a fresh God
    // pointing at the same home, and run resurrect.
    god.delete_selector(&Selector::All).await.expect("teardown");
    drop(god);

    let mut fresh = God::new(home);
    let started = fresh.resurrect().await.expect("resurrect");
    assert_eq!(started.len(), 1);
    assert_eq!(started[0].status, ProcessStatus::Online);
    let _ = fresh.stop_selector(&Selector::All).await;
}

/// PRD §9.1 row: `scale app N` → instances spawned per request.
#[tokio::test]
async fn start_app_with_three_instances() {
    let (_dir, home) = make_home();
    let mut god = God::new(home);
    let mut app = long_running_app("cluster");
    app.instances = InstanceCount::Count(3);
    let started = god.start_app(app).await.expect("start");
    assert_eq!(started.len(), 3);
    for info in started {
        assert_eq!(info.status, ProcessStatus::Online);
    }
    let _ = god.stop_selector(&Selector::All).await;
}

/// Selector::Name routes to all processes that share an app name.
#[tokio::test]
async fn selector_by_name_targets_named_app() {
    let (_dir, home) = make_home();
    let mut god = God::new(home);
    let mut app = long_running_app("dup");
    app.name = "named".into();
    god.start_app(app.clone()).await.expect("start 1");
    god.start_app(app).await.expect("start 2");

    let after = god
        .stop_selector(&Selector::Name("named".into()))
        .await
        .expect("stop by name");
    assert_eq!(after.len(), 2);
    for info in after {
        assert_eq!(info.status, ProcessStatus::Stopped);
        assert_eq!(info.name, "named");
    }
}

/// `send_signal` returns NotFound if no process matches the selector.
#[tokio::test]
async fn send_signal_to_missing_id_errors() {
    let (_dir, home) = make_home();
    let mut god = God::new(home);
    let err = god
        .send_signal(&Selector::Id(99), "SIGUSR1")
        .await
        .expect_err("missing");
    assert!(err.to_string().contains("99"), "msg was {err}");
}

/// Reload on fork-mode falls back to restart (returns Online + restart_time + 1).
#[tokio::test]
async fn reload_on_fork_mode_falls_back_to_restart() {
    let (_dir, home) = make_home();
    let mut god = God::new(home);
    god.start_app(long_running_app("forky"))
        .await
        .expect("start");
    let after = god.reload_selector(&Selector::All).await.expect("reload");
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].status, ProcessStatus::Online);
    // restart_time bumps for the fork-mode fallback path too.
    assert!(after[0].restart_time >= 1);
    let _ = god.stop_selector(&Selector::All).await;
}

/// Worker tick can be called repeatedly without panicking even on empty state.
#[tokio::test]
async fn worker_tick_is_idempotent_on_empty_state() {
    let (_dir, home) = make_home();
    let mut god = God::new(home);
    god.worker_tick().await.expect("tick 1");
    god.worker_tick().await.expect("tick 2");
    god.worker_tick().await.expect("tick 3");
}

/// Stop is fast: a well-behaved sleeper exits within kill_timeout.
#[tokio::test(flavor = "multi_thread")]
async fn stop_completes_within_kill_timeout() {
    let (_dir, home) = make_home();
    let mut god = God::new(home);
    god.start_app(long_running_app("speed"))
        .await
        .expect("start");

    let started = tokio::time::Instant::now();
    god.stop_selector(&Selector::All).await.expect("stop");
    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_secs(2),
        "stop took {elapsed:?}, expected < 2s"
    );
}
