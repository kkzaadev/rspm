//! IPC client.

use std::path::Path;

use tokio::net::UnixStream;

use rspm_core::{Result, RspmError};
use rspm_protocol::{Event, Request, Response};

use crate::codec::{read_frame, write_frame};

/// Request/reply IPC client.
#[derive(Debug)]
pub struct IpcClient {
    stream: UnixStream,
}

impl IpcClient {
    /// Connects to the daemon RPC socket.
    ///
    /// ```
    /// # async fn demo(path: &std::path::Path) -> rspm_core::Result<()> {
    /// let _client = rspm_ipc::IpcClient::connect(path).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(rpc_path: &Path) -> Result<Self> {
        let stream = UnixStream::connect(rpc_path).await?;
        Ok(Self { stream })
    }

    /// Sends one request and waits for one response.
    ///
    /// ```
    /// # async fn demo(mut client: rspm_ipc::IpcClient) -> rspm_core::Result<()> {
    /// let response = client.call(rspm_protocol::Request::Ping).await?;
    /// let _ = response.into_result()?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn call(&mut self, req: Request) -> Result<Response> {
        write_frame(&mut self.stream, &req)
            .await
            .map_err(|err| RspmError::Protocol(format!("failed to write request frame: {err}")))?;
        read_frame(&mut self.stream)
            .await
            .map_err(|err| RspmError::Protocol(format!("failed to read response frame: {err}")))
    }
}

/// Pub/sub subscriber. Reads framed [`Event`] values from `pub.sock` as they
/// are forwarded by the daemon.
#[derive(Debug)]
pub struct EventSubscriber {
    stream: UnixStream,
}

impl EventSubscriber {
    /// Connects to the daemon pub socket.
    ///
    /// ```
    /// # async fn demo(path: &std::path::Path) -> rspm_core::Result<()> {
    /// let _subscriber = rspm_ipc::EventSubscriber::connect(path).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(pub_path: &Path) -> Result<Self> {
        let stream = UnixStream::connect(pub_path).await?;
        Ok(Self { stream })
    }

    /// Returns the next published event, or `None` when the connection ends.
    pub async fn next_event(&mut self) -> Result<Option<Event>> {
        match read_frame::<_, Event>(&mut self.stream).await {
            Ok(event) => Ok(Some(event)),
            Err(RspmError::Io(err))
                if matches!(
                    err.kind(),
                    std::io::ErrorKind::UnexpectedEof
                        | std::io::ErrorKind::ConnectionReset
                        | std::io::ErrorKind::BrokenPipe
                ) =>
            {
                Ok(None)
            }
            Err(err) => Err(err),
        }
    }
}
