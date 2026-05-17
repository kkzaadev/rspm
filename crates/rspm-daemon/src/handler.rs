//! Request dispatch for the daemon.

use std::sync::Arc;

use tokio::sync::{Mutex, broadcast};

use rspm_core::version;
use rspm_protocol::{Request, Response};

use crate::god::God;

/// Dispatches one request to the daemon state.
pub async fn handle(
    god: Arc<Mutex<God>>,
    shutdown_tx: broadcast::Sender<()>,
    request: Request,
) -> Response {
    let result = match request {
        Request::Start { app } => {
            let mut god = god.lock().await;
            god.start_app(*app)
                .await
                .map(|processes| Response::Started { processes })
        }
        Request::Stop { selector } => {
            let mut god = god.lock().await;
            god.stop_selector(&selector)
                .await
                .map(|processes| Response::ProcessList { processes })
        }
        Request::Restart { selector } => {
            let mut god = god.lock().await;
            god.restart_selector(&selector)
                .await
                .map(|processes| Response::ProcessList { processes })
        }
        Request::Reload { selector } => {
            let mut god = god.lock().await;
            god.reload_selector(&selector)
                .await
                .map(|processes| Response::ProcessList { processes })
        }
        Request::Delete { selector } => {
            let mut god = god.lock().await;
            god.delete_selector(&selector)
                .await
                .map(|processes| Response::ProcessList { processes })
        }
        Request::List => {
            let mut god = god.lock().await;
            god.list()
                .await
                .map(|processes| Response::ProcessList { processes })
        }
        Request::Logs { selector, lines } => {
            let mut god = god.lock().await;
            god.logs(selector.as_ref(), lines)
                .await
                .map(|lines| Response::Logs { lines })
        }
        Request::Save => {
            let god = god.lock().await;
            god.save().await.map(|count| Response::Ack {
                message: format!("saved {count} processes"),
            })
        }
        Request::Resurrect => {
            let mut god = god.lock().await;
            god.resurrect()
                .await
                .map(|processes| Response::Started { processes })
        }
        Request::Ping => Ok(Response::Pong { msg: "pong".into() }),
        Request::GetVersion => Ok(Response::Version {
            version: version::pkg_version().to_owned(),
        }),
        Request::SendSignal { selector, signal } => {
            let mut god = god.lock().await;
            god.send_signal(&selector, &signal)
                .await
                .map(|processes| Response::ProcessList { processes })
        }
        Request::KillDaemon => {
            let mut god = god.lock().await;
            let stop_result = god.stop_all().await;
            let _ = shutdown_tx.send(());
            stop_result.map(|()| Response::Ack {
                message: "daemon stopped".to_owned(),
            })
        }
    };

    match result {
        Ok(response) => response,
        Err(err) => Response::error(err),
    }
}
