//! Mirror of `pm2/test/programmatic/logs.js`.
//!
//! Verifies the log-capture pipeline: every stdout line should reach the
//! per-app log file (rotation aware), and the bus should publish an
//! `Event::Log` for live subscribers.

use std::time::Duration;

use rspm_daemon::god::God;
use rspm_ipc::PubSubBus;
use rspm_protocol::{Event, LogStream, Selector};
use rspm_tests::{fixture_app, isolated_home};

#[tokio::test]
async fn stdout_lines_appear_in_out_log_file() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let app = fixture_app("logger", "log-many.sh");
    let started = god.start_app(app).await.expect("start");

    tokio::time::sleep(Duration::from_millis(500)).await;
    let out_path = started[0].out_file.clone().expect("out_file path");
    let body = tokio::fs::read_to_string(&out_path).await.expect("read");
    assert!(body.contains("line 0"));
    assert!(body.contains("line 49"));
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn bus_publishes_log_events_for_each_line() {
    let (_dir, home) = isolated_home();
    let bus = PubSubBus::new(256);
    let mut god = God::with_bus(home, bus.clone());
    let mut rx = bus.subscribe();

    god.start_app(fixture_app("logger", "log-many.sh"))
        .await
        .expect("start");

    let mut count = 0_u32;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    while tokio::time::Instant::now() < deadline && count < 50 {
        if let Ok(Ok(event)) = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await
            && matches!(
                event,
                Event::Log {
                    stream: LogStream::Out,
                    ..
                }
            )
        {
            count += 1;
        }
    }
    assert!(count >= 50, "expected >= 50 stdout log events, got {count}");
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn logs_method_returns_tail_lines() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    god.start_app(fixture_app("tail", "log-many.sh"))
        .await
        .expect("start");
    tokio::time::sleep(Duration::from_millis(500)).await;
    let lines = god.logs(Some(&Selector::All), 5).await.expect("logs");
    assert!(!lines.is_empty(), "expected some lines");
    assert!(
        lines.iter().any(|l| l.contains("line ")),
        "expected formatted lines, got: {lines:?}"
    );
    let _ = god.delete_selector(&Selector::All).await;
}
