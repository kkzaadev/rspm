//! IPC server.

use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

use tokio::net::{UnixListener, UnixStream};

use rspm_core::{Result, RspmError};
use rspm_protocol::{Event, Request, Response};

use crate::bus::PubSubBus;
use crate::codec::{read_frame, write_frame};

/// Boxed request handler future.
pub type BoxedRequestFuture = Pin<Box<dyn Future<Output = Response> + Send + 'static>>;
/// Shared async request handler.
pub type RequestHandler = Arc<dyn Fn(Request) -> BoxedRequestFuture + Send + Sync + 'static>;

/// Unix socket request/reply server.
#[derive(Debug)]
pub struct IpcServer {
    rpc_listener: UnixListener,
    pub_listener: UnixListener,
}

impl IpcServer {
    /// Binds RPC and pub/sub sockets.
    ///
    /// ```
    /// # fn demo(rpc: &std::path::Path, pubsock: &std::path::Path) -> rspm_core::Result<()> {
    /// let _server = rspm_ipc::IpcServer::bind(rpc, pubsock)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn bind(rpc_path: &Path, pub_path: &Path) -> Result<Self> {
        prepare_socket_path(rpc_path)?;
        prepare_socket_path(pub_path)?;
        let rpc_listener = UnixListener::bind(rpc_path)?;
        let pub_listener = UnixListener::bind(pub_path)?;
        Ok(Self {
            rpc_listener,
            pub_listener,
        })
    }

    /// Runs the accept loop until the shutdown future resolves. Subscribers
    /// on `pub_path` will be forwarded events from the optional `bus`.
    ///
    /// ```
    /// # async fn demo(server: rspm_ipc::IpcServer, handler: rspm_ipc::RequestHandler) -> rspm_core::Result<()> {
    /// server.run_until(handler, None, async {}).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run_until<F>(
        self,
        handler: RequestHandler,
        bus: Option<PubSubBus>,
        shutdown: F,
    ) -> Result<()>
    where
        F: Future<Output = ()> + Send,
    {
        let Self {
            rpc_listener,
            pub_listener,
        } = self;
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                _ = &mut shutdown => {
                    break;
                }
                accepted = rpc_listener.accept() => {
                    let (stream, _) = accepted?;
                    tracing::debug!("accepted ipc rpc connection");
                    let handler = Arc::clone(&handler);
                    tokio::spawn(async move {
                        if let Err(err) = handle_stream(stream, handler).await {
                            tracing::warn!(error = %err, "ipc request failed");
                        }
                    });
                }
                accepted = pub_listener.accept() => {
                    let (stream, _) = accepted?;
                    tracing::debug!("accepted ipc pub connection");
                    if let Some(bus) = bus.as_ref() {
                        let receiver = bus.subscribe();
                        tokio::spawn(async move {
                            if let Err(err) = forward_events(stream, receiver).await {
                                tracing::debug!(error = %err, "pub subscriber dropped");
                            }
                        });
                    }
                }
            }
        }

        Ok(())
    }
}

async fn forward_events(
    mut stream: UnixStream,
    mut receiver: tokio::sync::broadcast::Receiver<Event>,
) -> Result<()> {
    loop {
        match receiver.recv().await {
            Ok(event) => {
                write_frame(&mut stream, &event).await?;
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => return Ok(()),
            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                tracing::warn!(skipped, "pub subscriber lagged");
            }
        }
    }
}

async fn handle_stream(mut stream: UnixStream, handler: RequestHandler) -> Result<()> {
    loop {
        tracing::debug!("reading ipc request frame");
        let request: Request = match read_frame(&mut stream).await {
            Ok(request) => request,
            Err(RspmError::Io(err))
                if matches!(
                    err.kind(),
                    std::io::ErrorKind::UnexpectedEof
                        | std::io::ErrorKind::ConnectionReset
                        | std::io::ErrorKind::BrokenPipe
                ) =>
            {
                return Ok(());
            }
            Err(err) => return Err(err),
        };
        tracing::debug!(?request, "dispatching ipc request");
        let response = handler(request).await;
        write_frame(&mut stream, &response).await?;
    }
}

fn prepare_socket_path(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err.into()),
    }
}
