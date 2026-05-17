//! Mirror of `pm2/test/programmatic/client.mocha.js`.
//!
//! Exercises the public `rspm_ipc::IpcClient` + `IpcServer` contract used by
//! `rspm_client::RspmClient`. We boot a transient IPC server, fire common
//! requests, and confirm responses round-trip via the JSON frame codec.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use rspm_ipc::{IpcClient, IpcServer, RequestHandler};
use rspm_protocol::{Request, Response};

static N: AtomicU32 = AtomicU32::new(0);

fn tmp(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rspm-client-test-{}-{}-{label}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ))
}

fn stub_handler() -> RequestHandler {
    Arc::new(|request| {
        Box::pin(async move {
            match request {
                Request::Ping => Response::Pong { msg: "pong".into() },
                Request::List => Response::ProcessList {
                    processes: Vec::new(),
                },
                Request::GetVersion => Response::Version {
                    version: "test".into(),
                },
                _ => Response::error("unsupported"),
            }
        })
    })
}

#[tokio::test]
async fn client_ping_returns_pong() {
    let rpc = tmp("ping.sock");
    let pubsock = tmp("ping.pub.sock");
    let server = IpcServer::bind(&rpc, &pubsock).expect("bind");
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
    let task = tokio::spawn(async move {
        let _ = server
            .run_until(stub_handler(), None, async move {
                let _ = stop_rx.await;
            })
            .await;
    });
    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut client = IpcClient::connect(&rpc).await.expect("connect");
    let resp = client.call(Request::Ping).await.expect("ping");
    assert!(matches!(resp, Response::Pong { .. }));

    let _ = stop_tx.send(());
    let _ = task.await;
}

#[tokio::test]
async fn client_list_returns_process_list_envelope() {
    let rpc = tmp("list.sock");
    let pubsock = tmp("list.pub.sock");
    let server = IpcServer::bind(&rpc, &pubsock).expect("bind");
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
    let task = tokio::spawn(async move {
        let _ = server
            .run_until(stub_handler(), None, async move {
                let _ = stop_rx.await;
            })
            .await;
    });
    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut client = IpcClient::connect(&rpc).await.expect("connect");
    let resp = client.call(Request::List).await.expect("list");
    match resp {
        Response::ProcessList { processes } => assert!(processes.is_empty()),
        other => assert_eq!(format!("{other:?}").split('{').next(), Some("ProcessList ")),
    }

    let _ = stop_tx.send(());
    let _ = task.await;
}

#[tokio::test]
async fn client_unsupported_request_yields_error_envelope() {
    let rpc = tmp("err.sock");
    let pubsock = tmp("err.pub.sock");
    let server = IpcServer::bind(&rpc, &pubsock).expect("bind");
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
    let task = tokio::spawn(async move {
        let _ = server
            .run_until(stub_handler(), None, async move {
                let _ = stop_rx.await;
            })
            .await;
    });
    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut client = IpcClient::connect(&rpc).await.expect("connect");
    let resp = client.call(Request::Save).await.expect("save");
    assert!(matches!(resp, Response::Error { .. }));

    let _ = stop_tx.send(());
    let _ = task.await;
}
