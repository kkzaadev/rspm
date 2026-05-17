//! High-level daemon client.

use rspm_core::paths::RspmHome;
use rspm_core::{Result, RspmError};
use rspm_ipc::IpcClient;
use rspm_protocol::{Request, Response};

use crate::daemon_launcher::launch_if_needed;
use crate::reconnect::connect_with_retry;

/// Client connected to an RSPM daemon.
#[derive(Debug)]
pub struct RspmClient {
    home: RspmHome,
    ipc: IpcClient,
}

impl RspmClient {
    /// Connects to a daemon, launching it if needed.
    ///
    /// ```
    /// # async fn demo(home: rspm_core::paths::RspmHome) -> rspm_core::Result<()> {
    /// let _client = rspm_client::RspmClient::connect_or_launch(home).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect_or_launch(home: RspmHome) -> Result<Self> {
        home.ensure()?;

        if let Ok(client) = Self::connect(home.clone()).await {
            return Ok(client);
        }

        launch_if_needed(&home).await?;
        connect_with_retry(&home).await
    }

    /// Connects to an already running daemon.
    pub async fn connect(home: RspmHome) -> Result<Self> {
        let mut ipc = IpcClient::connect(&home.rpc_socket()).await?;
        let response = ipc.call(Request::Ping).await?;
        match response {
            Response::Pong { .. } => Ok(Self { home, ipc }),
            Response::Error { message } => Err(RspmError::Daemon(message)),
            other => Err(RspmError::Protocol(format!(
                "unexpected ping response: {other:?}"
            ))),
        }
    }

    /// Sends one raw protocol request.
    pub async fn call(&mut self, request: Request) -> Result<Response> {
        self.ipc.call(request).await?.into_result()
    }

    /// Returns this client's home directory.
    pub fn home(&self) -> &RspmHome {
        &self.home
    }
}
