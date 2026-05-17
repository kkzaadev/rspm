//! Mirror of `pm2/lib/API/Extra.js` describe / id / env / pid tests.
//!
//! PM2 exposes `pm2 describe`, `pm2 id`, `pm2 pid`, `pm2 env` as read-only
//! views over the existing process registry. These tests exercise the
//! daemon-side equivalents in [`rspm_daemon::god::God`].

use rspm_daemon::god::God;
use rspm_protocol::Selector;
use rspm_tests::{fixture_app, isolated_home};

#[tokio::test]
async fn describe_returns_metadata_for_running_process() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let mut app = fixture_app("desc", "sleeper.sh");
    app.max_restarts = 7;
    app.kill_timeout_ms = 4321;
    app.restart_delay_ms = 250;
    god.start_app(app).await.expect("start");

    let details = god.describe(&Selector::All).await.expect("describe");
    assert_eq!(details.len(), 1);
    let detail = &details[0];
    assert!(detail.info.name.starts_with("desc-"));
    assert_eq!(detail.max_restarts, 7);
    assert_eq!(detail.kill_timeout_ms, 4321);
    assert_eq!(detail.restart_delay_ms, 250);
    assert_eq!(detail.exec_mode, "fork_mode");
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn env_for_returns_effective_env_map() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let mut app = fixture_app("envapp", "sleeper.sh");
    app.env.insert("FOO".into(), "bar".into());
    god.start_app(app).await.expect("start");

    let envs = god.env_for(&Selector::All).await.expect("env_for");
    assert_eq!(envs.len(), 1);
    let env = envs.values().next().expect("one env map");
    assert_eq!(env.get("FOO").map(String::as_str), Some("bar"));
    assert!(
        env.get("name")
            .map(|name| name.starts_with("envapp-"))
            .unwrap_or(false)
    );
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn reset_counters_zeros_restart_time_and_unstable_restarts() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let mut app = fixture_app("resetapp", "sleeper.sh");
    app.max_restarts = 9999;
    god.start_app(app).await.expect("start");

    god.restart_selector(&Selector::All).await.expect("restart");
    god.restart_selector(&Selector::All).await.expect("restart");

    let after = god.reset_counters(&Selector::All).await.expect("reset");
    assert_eq!(after[0].restart_time, 0);
    assert_eq!(after[0].unstable_restarts, 0);
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn flush_truncates_log_files_for_running_process() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let app = fixture_app("flushapp", "log-many.sh");
    god.start_app(app).await.expect("start");
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;

    let count = god.flush(None).await.expect("flush");
    assert!(count >= 1, "expected at least one log truncation");
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn reload_logs_reopens_or_creates_log_sinks() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let app = fixture_app("rotapp", "sleeper.sh");
    god.start_app(app).await.expect("start");
    let count = god.reload_logs().await.expect("reload_logs");
    assert!(count >= 1);
    let _ = god.delete_selector(&Selector::All).await;
}
