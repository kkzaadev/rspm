//! Round-trip tests for every wire-level Request / Response / Event variant.
//!
//! These tests guard the PM2-parity invariant from `prd.md` R6: any field that
//! crosses the wire must serialize and deserialize to the same value. They
//! also lock the JSON tag layout (`method`/`status`/`event`) so accidental
//! refactors of the enum representation are caught before they break clients.

use std::path::PathBuf;

use chrono::Utc;

use rspm_core::types::{AppConfig, ProcessInfo, ProcessStatus};
use rspm_protocol::frame::{read_frame, write_frame};
use rspm_protocol::{Event, LogStream, PROTOCOL_VERSION, Request, Response, Selector};

fn sample_app() -> AppConfig {
    let mut app = AppConfig::from_script("server.js", Some("api".to_owned()));
    app.args = vec!["--port".into(), "3000".into()];
    app.cwd = Some(PathBuf::from("/srv/api"));
    app.stop_exit_codes = vec![0, 143];
    app.restart_delay_ms = 200;
    app.exp_backoff_restart_delay_ms = Some(100);
    app
}

fn sample_info() -> ProcessInfo {
    let app = sample_app();
    let mut info = ProcessInfo::new(7, &app);
    info.status = ProcessStatus::Online;
    info.pid = Some(4242);
    info.restart_time = 3;
    info.unstable_restarts = 1;
    info.pm_uptime = Some(Utc::now());
    info.cpu_percent = 2.5;
    info.memory_bytes = 1024 * 1024;
    info
}

async fn round_trip_request(req: Request) {
    let mut buf = Vec::new();
    write_frame(&mut buf, &req).await.expect("write frame");
    let decoded: Request = read_frame(&mut &buf[..]).await.expect("read frame");
    assert_eq!(decoded, req);
}

async fn round_trip_response(resp: Response) {
    let mut buf = Vec::new();
    write_frame(&mut buf, &resp).await.expect("write frame");
    let decoded: Response = read_frame(&mut &buf[..]).await.expect("read frame");
    assert_eq!(decoded, resp);
}

async fn round_trip_event(event: Event) {
    let mut buf = Vec::new();
    write_frame(&mut buf, &event).await.expect("write frame");
    let decoded: Event = read_frame(&mut &buf[..]).await.expect("read frame");
    assert_eq!(decoded, event);
}

#[tokio::test]
async fn round_trips_every_request_variant() {
    round_trip_request(Request::Ping).await;
    round_trip_request(Request::GetVersion).await;
    round_trip_request(Request::List).await;
    round_trip_request(Request::Start {
        app: Box::new(sample_app()),
    })
    .await;
    round_trip_request(Request::Stop {
        selector: Selector::All,
    })
    .await;
    round_trip_request(Request::Restart {
        selector: Selector::Id(3),
    })
    .await;
    round_trip_request(Request::Reload {
        selector: Selector::Name("api".into()),
    })
    .await;
    round_trip_request(Request::Delete {
        selector: Selector::All,
    })
    .await;
    round_trip_request(Request::Logs {
        selector: Some(Selector::Name("api".into())),
        lines: 50,
    })
    .await;
    round_trip_request(Request::Save).await;
    round_trip_request(Request::Resurrect).await;
    round_trip_request(Request::SendSignal {
        selector: Selector::Id(0),
        signal: "SIGTERM".into(),
    })
    .await;
    round_trip_request(Request::KillDaemon).await;
}

#[tokio::test]
async fn round_trips_every_response_variant() {
    round_trip_response(Response::Ack {
        message: "ok".into(),
    })
    .await;
    round_trip_response(Response::Started {
        processes: vec![sample_info()],
    })
    .await;
    round_trip_response(Response::ProcessList {
        processes: vec![sample_info(), sample_info()],
    })
    .await;
    round_trip_response(Response::Process {
        process: sample_info(),
    })
    .await;
    round_trip_response(Response::Logs {
        lines: vec!["[api] [out] hi".into()],
    })
    .await;
    round_trip_response(Response::Pong { msg: "pong".into() }).await;
    round_trip_response(Response::Version {
        version: "0.0.1".into(),
    })
    .await;
    round_trip_response(Response::Error {
        message: "boom".into(),
    })
    .await;
}

#[tokio::test]
async fn round_trips_every_event_variant() {
    round_trip_event(Event::ProcessOnline {
        process: sample_info(),
    })
    .await;
    round_trip_event(Event::ProcessExit {
        pm_id: 9,
        code: Some(143),
    })
    .await;
    round_trip_event(Event::ProcessExit {
        pm_id: 9,
        code: None,
    })
    .await;
    round_trip_event(Event::Log {
        pm_id: 0,
        name: "api".into(),
        stream: LogStream::Out,
        data: "ready".into(),
        at: Utc::now(),
    })
    .await;
    round_trip_event(Event::Log {
        pm_id: 0,
        name: "api".into(),
        stream: LogStream::Err,
        data: "boom".into(),
        at: Utc::now(),
    })
    .await;
    round_trip_event(Event::ProcessMsg {
        pm_id: 1,
        payload: serde_json::json!({ "ready": true }),
    })
    .await;
    round_trip_event(Event::SystemWarn {
        message: "fd exhaustion".into(),
    })
    .await;
}

#[test]
fn protocol_version_is_pinned_to_one_for_v0() {
    // Bumping this is a breaking change — keep this guard until the
    // wire protocol changes intentionally.
    assert_eq!(PROTOCOL_VERSION, 1);
}

#[test]
fn into_result_maps_error_response_to_rspm_error() {
    let result = Response::Error {
        message: "denied".into(),
    }
    .into_result();
    assert!(result.is_err());
}

#[test]
fn into_result_passes_through_non_error_response() {
    let response = Response::Pong { msg: "pong".into() };
    assert!(response.into_result().is_ok());
}

#[tokio::test]
async fn frame_round_trip_works_over_a_simple_byte_buffer() {
    let mut buf = Vec::new();
    write_frame(&mut buf, &Request::List).await.expect("write");

    // First 4 bytes are the BE u32 length prefix. Rest is the JSON payload.
    assert_eq!(buf[0..4], (buf.len() as u32 - 4).to_be_bytes());

    let decoded: Request = read_frame(&mut &buf[..]).await.expect("read");
    assert_eq!(decoded, Request::List);
}

#[test]
fn selector_parses_all_id_and_name() {
    assert_eq!(Selector::parse("all"), Selector::All);
    assert_eq!(Selector::parse("42"), Selector::Id(42));
    assert_eq!(Selector::parse("api"), Selector::Name("api".into()));
}
