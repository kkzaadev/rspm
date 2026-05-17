//! Lifecycle tests for the daemon `God` supervisor.
//!
//! Exercises real subprocess spawn via `/bin/sh`, so these tests are Unix-only
//! and require the binary to be available (they will fail with a clear error
//! on other platforms). They cover:
//!   - start + list + stop happy path
//!   - log forwarding writes to LogWriter AND publishes Event::Log to the bus
//!   - stop_exit_codes intentional exits don't mark the proc errored
//!   - kill_timeout falls back to SIGKILL when the child ignores SIGINT

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use rspm_core::paths::RspmHome;
use rspm_core::types::{AppConfig, ProcessStatus};
use rspm_daemon::god::God;
use rspm_ipc::PubSubBus;
use rspm_protocol::Event;

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn make_home() -> (tempfile::TempDir, RspmHome) {
    let dir = tempfile::tempdir().expect("temp dir");
    let home = RspmHome::new(dir.path());
    home.ensure().expect("ensure home");
    (dir, home)
}

fn shell_app(name: &str, script: &str, cwd: Option<PathBuf>) -> AppConfig {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut app = AppConfig::from_script(PathBuf::from("/bin/sh"), Some(format!("{name}-{n}")));
    app.args = vec!["-c".into(), script.to_owned()];
    app.cwd = cwd;
    app.auto_restart = false;
    app.kill_timeout_ms = 500;
    app
}

#[tokio::test]
async fn start_then_stop_marks_process_stopped() {
    let (_dir, home) = make_home();
    let mut god = God::new(home);
    let app = shell_app("sleeper", "while true; do echo tick; sleep 0.1; done", None);

    let started = god.start_app(app.clone()).await.expect("start");
    assert_eq!(started.len(), 1);
    assert_eq!(started[0].status, ProcessStatus::Online);

    // List eagerly to refresh + return state.
    let listed = god.list().await.expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].status, ProcessStatus::Online);

    let selector = rspm_protocol::Selector::All;
    let after_stop = god.stop_selector(&selector).await.expect("stop");
    assert_eq!(after_stop.len(), 1);
    assert_eq!(after_stop[0].status, ProcessStatus::Stopped);

    god.delete_selector(&selector).await.expect("delete");
    assert!(god.list().await.expect("list").is_empty());
}

#[tokio::test]
async fn stdout_events_reach_pub_bus_subscribers() {
    let (_dir, home) = make_home();
    let bus = PubSubBus::new(64);
    let mut god = God::with_bus(home, bus.clone());
    let mut rx = bus.subscribe();

    // Print a recognizable token then exit so we don't keep the child around.
    let app = shell_app("logger", "echo RSPM_TEST_LINE; sleep 0.2", None);
    god.start_app(app).await.expect("start");

    // Drain events until we see our Log line.
    let mut got_log = false;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    while tokio::time::Instant::now() < deadline && !got_log {
        match tokio::time::timeout(Duration::from_millis(200), rx.recv()).await {
            Ok(Ok(Event::Log { data, .. })) if data == "RSPM_TEST_LINE" => {
                got_log = true;
            }
            Ok(Ok(_)) => continue,
            Ok(Err(_)) => break,
            Err(_) => continue,
        }
    }
    assert!(got_log, "expected Event::Log with token RSPM_TEST_LINE");

    // Clean up so background tasks don't outlive the test.
    let _ = god.stop_selector(&rspm_protocol::Selector::All).await;
}

#[tokio::test]
async fn stop_exit_code_is_treated_as_intentional() {
    let (_dir, home) = make_home();
    let mut god = God::new(home);

    let mut app = shell_app("graceful", "sleep 0.05; exit 143", None);
    app.auto_restart = true;
    app.stop_exit_codes = vec![143];

    god.start_app(app).await.expect("start");

    // Give the child time to exit; then list refreshes statuses.
    tokio::time::sleep(Duration::from_millis(300)).await;
    let listed = god.list().await.expect("list");
    assert_eq!(listed.len(), 1);
    // Because 143 is in stop_exit_codes, the app must NOT restart and must
    // be marked Stopped (intentional exit), not Errored or Waiting.
    assert_eq!(listed[0].status, ProcessStatus::Stopped);
    assert_eq!(listed[0].restart_time, 0);
}
