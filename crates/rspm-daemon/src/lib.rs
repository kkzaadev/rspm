//! RSPM daemon and supervisor.

pub mod god;
pub mod handler;
pub mod handlers;
pub mod log_capture;
pub mod pid_file;
pub mod supervisor;
pub mod worker;

use std::sync::Arc;

use tokio::sync::{Mutex, broadcast};

use rspm_core::Result;
use rspm_core::paths::RspmHome;
use rspm_ipc::{IpcServer, RequestHandler};

use crate::god::God;

/// Runs the daemon until a shutdown request or signal is received.
///
/// ```
/// # async fn demo(home: rspm_core::paths::RspmHome) -> rspm_core::Result<()> {
/// # let _ = home;
/// # Ok(())
/// # }
/// ```
pub async fn run(home: RspmHome) -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    home.ensure()?;
    pid_file::write_pid(&home)?;

    let server = IpcServer::bind(&home.rpc_socket(), &home.pub_socket())?;
    let bus = rspm_ipc::PubSubBus::new(rspm_core::defaults::pub_bus_capacity());
    let mut state = God::with_bus(home.clone(), bus.clone());
    let restart_rx = state.take_restart_rx();
    let god = Arc::new(Mutex::new(state));
    let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(4);
    spawn_signal_shutdown(shutdown_tx.clone());

    let worker_handle = {
        let god = Arc::clone(&god);
        let worker_rx = shutdown_tx.subscribe();
        tokio::spawn(async move {
            worker::run_until(god, worker_rx).await;
        })
    };

    let restart_handle = restart_rx.map(|mut rx| {
        let god = Arc::clone(&god);
        let mut shutdown_rx = shutdown_tx.subscribe();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    pm_id = rx.recv() => match pm_id {
                        Some(pm_id) => {
                            let mut god = god.lock().await;
                            if let Err(err) = god.restart_by_id(pm_id).await {
                                tracing::warn!(pm_id, error = %err, "watcher restart failed");
                            }
                        }
                        None => break,
                    },
                    _ = shutdown_rx.recv() => break,
                }
            }
        })
    });

    let handler: RequestHandler = {
        let god = Arc::clone(&god);
        let shutdown_tx = shutdown_tx.clone();
        Arc::new(move |request| {
            let god = Arc::clone(&god);
            let shutdown_tx = shutdown_tx.clone();
            Box::pin(async move { handler::handle(god, shutdown_tx, request).await })
        })
    };

    server
        .run_until(handler, Some(bus), async move {
            let _ = shutdown_rx.recv().await;
        })
        .await?;

    let _ = worker_handle.await;
    if let Some(handle) = restart_handle {
        let _ = handle.await;
    }

    let mut god = god.lock().await;
    god.stop_all().await?;
    cleanup_home(&home);
    Ok(())
}

fn spawn_signal_shutdown(shutdown_tx: broadcast::Sender<()>) {
    tokio::spawn(async move {
        #[cfg(unix)]
        {
            let mut sigterm =
                match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                    Ok(signal) => signal,
                    Err(_) => {
                        let _ = tokio::signal::ctrl_c().await;
                        let _ = shutdown_tx.send(());
                        return;
                    }
                };

            tokio::select! {
                _ = tokio::signal::ctrl_c() => {},
                _ = sigterm.recv() => {},
            }
        }

        #[cfg(not(unix))]
        {
            let _ = tokio::signal::ctrl_c().await;
        }

        let _ = shutdown_tx.send(());
    });
}

fn cleanup_home(home: &RspmHome) {
    let _ = std::fs::remove_file(home.rpc_socket());
    let _ = std::fs::remove_file(home.pub_socket());
    let _ = std::fs::remove_file(home.daemon_pid_file());
}
