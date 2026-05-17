//! Typed client API.

use std::collections::BTreeMap;

use rspm_core::types::{AppConfig, ProcessId, ProcessInfo};
use rspm_core::{Result, RspmError};
use rspm_protocol::{ProcessDetail, Request, Response, Selector};

use crate::client::RspmClient;

impl RspmClient {
    /// Starts an app.
    pub async fn start_app(&mut self, app: AppConfig) -> Result<Vec<ProcessInfo>> {
        match self.call(Request::Start { app: Box::new(app) }).await? {
            Response::Started { processes } => Ok(processes),
            other => Err(unexpected(other)),
        }
    }

    /// Lists processes.
    pub async fn list(&mut self) -> Result<Vec<ProcessInfo>> {
        match self.call(Request::List).await? {
            Response::ProcessList { processes } => Ok(processes),
            other => Err(unexpected(other)),
        }
    }

    /// Stops processes.
    pub async fn stop(&mut self, selector: Selector) -> Result<Vec<ProcessInfo>> {
        match self.call(Request::Stop { selector }).await? {
            Response::ProcessList { processes } => Ok(processes),
            other => Err(unexpected(other)),
        }
    }

    /// Restarts processes.
    pub async fn restart(&mut self, selector: Selector) -> Result<Vec<ProcessInfo>> {
        match self.call(Request::Restart { selector }).await? {
            Response::ProcessList { processes } => Ok(processes),
            other => Err(unexpected(other)),
        }
    }

    /// Soft-reloads processes (zero-downtime rolling restart in cluster mode).
    pub async fn reload(&mut self, selector: Selector) -> Result<Vec<ProcessInfo>> {
        match self.call(Request::Reload { selector }).await? {
            Response::ProcessList { processes } => Ok(processes),
            other => Err(unexpected(other)),
        }
    }

    /// Deletes processes.
    pub async fn delete(&mut self, selector: Selector) -> Result<Vec<ProcessInfo>> {
        match self.call(Request::Delete { selector }).await? {
            Response::ProcessList { processes } => Ok(processes),
            other => Err(unexpected(other)),
        }
    }

    /// Reads log tail lines.
    pub async fn logs(&mut self, selector: Option<Selector>, lines: usize) -> Result<Vec<String>> {
        match self.call(Request::Logs { selector, lines }).await? {
            Response::Logs { lines } => Ok(lines),
            other => Err(unexpected(other)),
        }
    }

    /// Saves process list.
    pub async fn save(&mut self) -> Result<String> {
        match self.call(Request::Save).await? {
            Response::Ack { message } => Ok(message),
            other => Err(unexpected(other)),
        }
    }

    /// Resurrects apps from the dump file.
    pub async fn resurrect(&mut self) -> Result<Vec<ProcessInfo>> {
        match self.call(Request::Resurrect).await? {
            Response::Started { processes } => Ok(processes),
            other => Err(unexpected(other)),
        }
    }

    /// Pings the daemon.
    pub async fn ping(&mut self) -> Result<String> {
        match self.call(Request::Ping).await? {
            Response::Pong { msg } => Ok(msg),
            other => Err(unexpected(other)),
        }
    }

    /// Sends a signal to processes.
    pub async fn send_signal(
        &mut self,
        selector: Selector,
        signal: String,
    ) -> Result<Vec<ProcessInfo>> {
        match self.call(Request::SendSignal { selector, signal }).await? {
            Response::ProcessList { processes } => Ok(processes),
            other => Err(unexpected(other)),
        }
    }

    /// Kills the daemon.
    pub async fn kill_daemon(&mut self) -> Result<String> {
        match self.call(Request::KillDaemon).await? {
            Response::Ack { message } => Ok(message),
            other => Err(unexpected(other)),
        }
    }

    /// Returns detailed metadata for matching processes (`rspm describe <id>`).
    pub async fn describe(&mut self, selector: Selector) -> Result<Vec<ProcessDetail>> {
        match self.call(Request::Describe { selector }).await? {
            Response::Describe { details } => Ok(details),
            other => Err(unexpected(other)),
        }
    }

    /// Returns effective env per process (`rspm env <id>`).
    pub async fn env_for(
        &mut self,
        selector: Selector,
    ) -> Result<BTreeMap<ProcessId, BTreeMap<String, String>>> {
        match self.call(Request::Env { selector }).await? {
            Response::Env { envs } => Ok(envs),
            other => Err(unexpected(other)),
        }
    }

    /// Resizes a cluster-mode app to `instances` (`rspm scale <name> <n>`).
    pub async fn scale(&mut self, name: String, instances: u32) -> Result<Vec<ProcessInfo>> {
        match self.call(Request::Scale { name, instances }).await? {
            Response::ProcessList { processes } => Ok(processes),
            other => Err(unexpected(other)),
        }
    }

    /// Truncates log files for matching processes (`rspm flush`).
    pub async fn flush(&mut self, selector: Option<Selector>) -> Result<String> {
        match self.call(Request::Flush { selector }).await? {
            Response::Ack { message } => Ok(message),
            other => Err(unexpected(other)),
        }
    }

    /// Zeroes restart counters (`rspm reset <id>`).
    pub async fn reset(&mut self, selector: Selector) -> Result<Vec<ProcessInfo>> {
        match self.call(Request::Reset { selector }).await? {
            Response::ProcessList { processes } => Ok(processes),
            other => Err(unexpected(other)),
        }
    }

    /// Reopens log file descriptors (`rspm reloadLogs`).
    pub async fn reload_logs(&mut self) -> Result<String> {
        match self.call(Request::ReloadLogs).await? {
            Response::Ack { message } => Ok(message),
            other => Err(unexpected(other)),
        }
    }
}

fn unexpected(response: Response) -> RspmError {
    RspmError::Protocol(format!("unexpected response: {response:?}"))
}
