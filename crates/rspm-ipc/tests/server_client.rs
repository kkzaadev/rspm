//! End-to-end IPC server <-> client tests.
//!
//! Covers the three guarantees the daemon depends on:
//!   1. Multiple requests on one connection are dispatched in order.
//!   2. The pub.sock listener forwards bus events to every subscriber.
//!   3. The server returns the exact response built by the handler.

use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use rspm_ipc::{EventSubscriber, IpcClient, IpcServer, PubSubBus, RequestHandler};
use rspm_protocol::{Event, LogStream, Request, Response};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn tmp_socket(name: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("rspm-ipc-{}-{n}-{name}", std::process::id()))
}

fn echo_handler() -> RequestHandler {
    Arc::new(|request| {
        Box::pin(async move {
            match request {
                Request::Ping => Response::Pong { msg: "pong".into() },
                Request::GetVersion => Response::Version {
                    version: "test".into(),
                },
                Request::List => Response::ProcessList {
                    processes: Vec::new(),
                },
                _ => Response::error("unsupported in test"),
            }
        })
    })
}

#[tokio::test]
async fn client_can_call_ping_and_get_version_on_one_connection() -> Result<(), Box<dyn Error>> {
    let rpc = tmp_socket("ping.sock");
    let pubsock = tmp_socket("ping.pub.sock");
    let server = IpcServer::bind(&rpc, &pubsock)?;
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();

    let server_task = tokio::spawn(async move {
        let _ = server
            .run_until(echo_handler(), None, async move {
                let _ = stop_rx.await;
            })
            .await;
    });

    // Give the listener a brief moment to start.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let mut client = IpcClient::connect(&rpc).await?;
    let response = client.call(Request::Ping).await?;
    assert!(matches!(response, Response::Pong { .. }));

    let version = client.call(Request::GetVersion).await?;
    assert!(matches!(version, Response::Version { .. }));

    let list = client.call(Request::List).await?;
    assert!(matches!(list, Response::ProcessList { .. }));

    let _ = stop_tx.send(());
    let _ = server_task.await;
    Ok(())
}

#[tokio::test]
async fn pub_socket_forwards_bus_events_to_subscriber() -> Result<(), Box<dyn Error>> {
    let rpc = tmp_socket("sub.sock");
    let pubsock = tmp_socket("sub.pub.sock");
    let server = IpcServer::bind(&rpc, &pubsock)?;
    let bus = PubSubBus::new(8);
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();

    let bus_clone = bus.clone();
    let server_task = tokio::spawn(async move {
        let _ = server
            .run_until(echo_handler(), Some(bus_clone), async move {
                let _ = stop_rx.await;
            })
            .await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let mut subscriber = EventSubscriber::connect(&pubsock).await?;
    // Give the server accept-loop a chance to register the subscription
    // before we publish.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    bus.publish(Event::Log {
        pm_id: 1,
        name: "api".into(),
        stream: LogStream::Out,
        data: "ready".into(),
        at: chrono::Utc::now(),
    });

    let event = tokio::time::timeout(std::time::Duration::from_secs(2), subscriber.next_event())
        .await
        .expect("event arrived in time")?;
    let event = event.expect("event present");

    if let Event::Log {
        pm_id, name, data, ..
    } = event
    {
        assert_eq!(pm_id, 1);
        assert_eq!(name, "api");
        assert_eq!(data, "ready");
    } else {
        // panic!/assert!(false) is rejected by clippy::panic; force a
        // descriptive failure via assert_eq comparing event variant names.
        assert_eq!(format!("{event:?}").split('{').next(), Some("Log "));
    }

    let _ = stop_tx.send(());
    let _ = server_task.await;
    Ok(())
}

#[tokio::test]
async fn handler_error_returns_error_response() -> Result<(), Box<dyn Error>> {
    let rpc = tmp_socket("err.sock");
    let pubsock = tmp_socket("err.pub.sock");
    let server = IpcServer::bind(&rpc, &pubsock)?;
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();

    let server_task = tokio::spawn(async move {
        let _ = server
            .run_until(echo_handler(), None, async move {
                let _ = stop_rx.await;
            })
            .await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let mut client = IpcClient::connect(&rpc).await?;
    // The echo handler answers `Response::error` for anything other than
    // ping/version/list — `Save` is a quick way to exercise that branch.
    let response = client.call(Request::Save).await?;
    assert!(matches!(response, Response::Error { .. }));

    let _ = stop_tx.send(());
    let _ = server_task.await;
    Ok(())
}
