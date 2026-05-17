//! PM2-style `God` supervisor state.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::str::FromStr;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use chrono::{DateTime, Utc};
use cron::Schedule;
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStderr, ChildStdout, Command};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::timeout;

use rspm_core::paths::{RspmHome, path_to_string};
use rspm_core::types::{
    AppConfig, ExecutionMode, ProcessId, ProcessInfo, ProcessStatus, WatchSpec,
};
use rspm_core::{Result, RspmError};
use rspm_ipc::PubSubBus;
use rspm_logs::{LogOpts, LogWriter, tail_file};
use rspm_monitor::{Aggregator, Sampler};
use rspm_protocol::{Event, LogStream, Selector};
use rspm_watcher::AppWatcher;

#[derive(Debug)]
struct ManagedProcess {
    app: AppConfig,
    info: ProcessInfo,
    child: Option<Child>,
    instance_index: u32,
    watcher: Option<JoinHandle<()>>,
    /// Handles to the per-stream log-forwarder tasks; aborted on stop so
    /// they don't keep file descriptors open against a dead child.
    log_tasks: Vec<JoinHandle<()>>,
    /// Current exponential backoff delay (ms). Reset to 0 by the worker tick
    /// once the process has been stable for [`rspm_core::defaults::EXP_BACKOFF_RESET_TIMER_MS`].
    prev_restart_delay_ms: u64,
    /// Instant the daemon may attempt the next restart for this process.
    /// Used by `refresh_and_autorestart` to honor `restart_delay` and the
    /// exponential backoff sequence.
    next_restart_at: Option<tokio::time::Instant>,
}

impl ManagedProcess {
    fn abort_watcher(&mut self) {
        if let Some(handle) = self.watcher.take() {
            handle.abort();
        }
    }

    fn abort_log_tasks(&mut self) {
        for handle in self.log_tasks.drain(..) {
            handle.abort();
        }
    }
}

/// Daemon supervisor state.
#[derive(Debug)]
pub struct God {
    home: RspmHome,
    next_id: ProcessId,
    processes: BTreeMap<ProcessId, ManagedProcess>,
    sampler: Sampler,
    aggregator: Aggregator,
    cron_next: HashMap<ProcessId, DateTime<Utc>>,
    restart_tx: mpsc::UnboundedSender<ProcessId>,
    restart_rx: Option<mpsc::UnboundedReceiver<ProcessId>>,
    bus: PubSubBus,
}

impl God {
    /// Creates a new supervisor state without a wired event bus. Mostly for
    /// tests that don't care about pub/sub events.
    pub fn new(home: RspmHome) -> Self {
        Self::with_bus(
            home,
            PubSubBus::new(rspm_core::defaults::pub_bus_capacity()),
        )
    }

    /// Creates a new supervisor state with an externally-provided event bus
    /// so the daemon `lib::run` can forward events to subscribers on `pub.sock`.
    pub fn with_bus(home: RspmHome, bus: PubSubBus) -> Self {
        let (restart_tx, restart_rx) = mpsc::unbounded_channel();
        Self {
            home,
            next_id: 0,
            processes: BTreeMap::new(),
            sampler: Sampler::new(rspm_core::defaults::worker_interval()),
            aggregator: Aggregator::new(rspm_monitor::aggregator::DEFAULT_WINDOW),
            cron_next: HashMap::new(),
            restart_tx,
            restart_rx: Some(restart_rx),
            bus,
        }
    }

    /// Returns a clone of the event bus for external subscribers (the
    /// pub.sock forwarder in `lib::run`).
    pub fn bus(&self) -> PubSubBus {
        self.bus.clone()
    }

    /// Takes the receiver used to deliver watcher-triggered restart requests.
    /// Returns `None` after the first call so the runtime can guarantee a
    /// single consumer.
    pub fn take_restart_rx(&mut self) -> Option<mpsc::UnboundedReceiver<ProcessId>> {
        self.restart_rx.take()
    }

    /// Restarts a process by id, ignoring errors when the id has been deleted.
    pub async fn restart_by_id(&mut self, pm_id: ProcessId) -> Result<()> {
        if !self.processes.contains_key(&pm_id) {
            return Ok(());
        }
        self.restart_id(pm_id).await?;
        Ok(())
    }

    /// Starts an application and returns the created processes.
    pub async fn start_app(&mut self, app: AppConfig) -> Result<Vec<ProcessInfo>> {
        let count = app.instances.resolve();
        let mut started = Vec::new();

        for instance_index in 0..count {
            let pm_id = self.allocate_id();
            let mut info = ProcessInfo::new(pm_id, &app);
            info.status = ProcessStatus::Launching;
            self.processes.insert(
                pm_id,
                ManagedProcess {
                    app: app.clone(),
                    info,
                    child: None,
                    instance_index,
                    watcher: None,
                    log_tasks: Vec::new(),
                    prev_restart_delay_ms: 0,
                    next_restart_at: None,
                },
            );
            started.push(self.spawn_for_id(pm_id).await?);
            self.ensure_watcher(pm_id);
        }

        Ok(started)
    }

    /// Spawns a file watcher for `pm_id` if its app opts in. Idempotent: a
    /// watcher is only created when one is not already running.
    fn ensure_watcher(&mut self, pm_id: ProcessId) {
        let Some(managed) = self.processes.get(&pm_id) else {
            return;
        };
        if managed.watcher.is_some() {
            return;
        }
        if !watch_enabled(&managed.app.watch) {
            return;
        }
        let cwd = watcher_root(&managed.app);
        let watch = managed.app.watch.clone();
        let ignore = managed.app.ignore_watch.clone();
        let tx = self.restart_tx.clone();

        match AppWatcher::new(&cwd, watch, &ignore) {
            Ok(watcher) => {
                if watcher.is_disabled() {
                    return;
                }
                let handle = tokio::spawn(run_watcher(watcher, pm_id, tx));
                if let Some(managed) = self.processes.get_mut(&pm_id) {
                    managed.watcher = Some(handle);
                }
            }
            Err(err) => {
                tracing::warn!(pm_id, error = %err, "watcher disabled (init failed)");
            }
        }
    }

    /// Stops every managed process.
    pub async fn stop_all(&mut self) -> Result<()> {
        let ids = self.processes.keys().copied().collect::<Vec<_>>();
        for pm_id in ids {
            self.stop_id(pm_id).await?;
        }
        Ok(())
    }

    /// Stops matching processes.
    pub async fn stop_selector(&mut self, selector: &Selector) -> Result<Vec<ProcessInfo>> {
        let ids = self.select_ids(selector)?;
        for pm_id in ids {
            self.stop_id(pm_id).await?;
        }
        self.list().await
    }

    /// Restarts matching processes.
    pub async fn restart_selector(&mut self, selector: &Selector) -> Result<Vec<ProcessInfo>> {
        let ids = self.select_ids(selector)?;
        for pm_id in ids {
            self.restart_id(pm_id).await?;
        }
        self.list().await
    }

    /// Soft-reload matching processes one at a time.
    ///
    /// For fork-mode apps this is equivalent to `restart` (the child is
    /// stopped then respawned). For cluster-mode apps it performs a rolling
    /// reload: spawn the replacement first, wait until it reports ready
    /// (T12.4 wait_ready / listen_timeout), then stop the previous instance
    /// so connections keep being served by at least one healthy process.
    /// Mirrors `pm2/lib/God/Reload.js`.
    pub async fn reload_selector(&mut self, selector: &Selector) -> Result<Vec<ProcessInfo>> {
        let ids = self.select_ids(selector)?;
        for pm_id in ids {
            self.reload_id(pm_id).await?;
        }
        self.list().await
    }

    async fn reload_id(&mut self, pm_id: ProcessId) -> Result<ProcessInfo> {
        let (app, instance_index) = match self.processes.get(&pm_id) {
            Some(managed) => (managed.app.clone(), managed.instance_index),
            None => return Err(RspmError::NotFound(format!("process id {pm_id}"))),
        };

        // Only cluster mode supports true zero-downtime reload because two
        // children must temporarily share the listening port (SO_REUSEPORT).
        // Fork mode falls back to a regular restart.
        if !matches!(app.execution_mode, ExecutionMode::ClusterMode) {
            return self.restart_id(pm_id).await;
        }

        // Allocate a brand-new id for the replacement so the old instance can
        // keep running concurrently. The new instance inherits the same
        // instance_index so SO_REUSEPORT routing remains identical.
        let new_id = self.allocate_id();
        let mut new_info = ProcessInfo::new(new_id, &app);
        new_info.status = ProcessStatus::Launching;
        self.processes.insert(
            new_id,
            ManagedProcess {
                app: app.clone(),
                info: new_info,
                child: None,
                instance_index,
                watcher: None,
                log_tasks: Vec::new(),
                prev_restart_delay_ms: 0,
                next_restart_at: None,
            },
        );
        let new_started = self.spawn_for_id(new_id).await?;

        // Wait for the new instance to declare readiness (T12.4 sentinel file)
        // before we tear down the previous one.
        wait_for_ready(&self.home, &app, new_id).await;

        // Stop the previous instance and remove it from state.
        self.stop_id(pm_id).await?;
        if let Some(pid) = self.processes.remove(&pm_id).and_then(|m| m.info.pid) {
            self.aggregator.forget(pid);
        }
        self.cron_next.remove(&pm_id);

        // Arm watcher on the new instance.
        self.ensure_watcher(new_id);
        Ok(new_started)
    }

    /// Deletes matching processes from daemon state.
    pub async fn delete_selector(&mut self, selector: &Selector) -> Result<Vec<ProcessInfo>> {
        let ids = self.select_ids(selector)?;
        for pm_id in ids {
            self.stop_id(pm_id).await?;
            if let Some(pid) = self.processes.remove(&pm_id).and_then(|m| m.info.pid) {
                self.aggregator.forget(pid);
            }
            self.cron_next.remove(&pm_id);
        }
        if self.processes.is_empty() {
            self.next_id = 0;
        }
        self.list().await
    }

    /// Lists process state.
    pub async fn list(&mut self) -> Result<Vec<ProcessInfo>> {
        self.refresh_and_autorestart().await?;
        Ok(self
            .processes
            .values()
            .map(|managed| managed.info.clone())
            .collect())
    }

    /// Runs one worker tick: refresh statuses, sample metrics, then enforce
    /// memory and cron policies. Mirrors `pm2/lib/Worker.js` `tasks()`.
    pub async fn worker_tick(&mut self) -> Result<()> {
        self.refresh_and_autorestart().await?;
        self.reset_stable_backoff();
        self.sample_metrics();
        self.check_memory_limits().await?;
        self.run_cron_restarts().await?;
        Ok(())
    }

    /// Samples cpu/mem for every running pid and updates `ProcessInfo`.
    fn sample_metrics(&mut self) {
        let pids: Vec<u32> = self
            .processes
            .values()
            .filter_map(|managed| managed.info.pid)
            .collect();
        let samples = self.sampler.sample(&pids);
        for (pid, cpu, mem) in samples {
            self.aggregator.record(pid, cpu.percent, mem.bytes);
            if let Some(managed) = self
                .processes
                .values_mut()
                .find(|m| m.info.pid == Some(pid))
            {
                if let Some((avg_cpu, avg_mem)) = self.aggregator.average(pid) {
                    managed.info.cpu_percent = avg_cpu;
                    managed.info.memory_bytes = avg_mem;
                } else {
                    managed.info.cpu_percent = cpu.percent;
                    managed.info.memory_bytes = mem.bytes;
                }
            }
        }
    }

    /// Restarts processes whose RSS exceeds `max_memory_restart`.
    ///
    /// Matches `pm2/lib/Worker.js` `maxMemoryRestart`: when a running process
    /// is over the configured threshold, restart it. PM2 uses `reload` for
    /// cluster apps and `restart` for fork apps — until T12 lands soft reload
    /// we always restart.
    pub async fn check_memory_limits(&mut self) -> Result<()> {
        let targets: Vec<ProcessId> = self
            .processes
            .iter()
            .filter_map(|(pm_id, managed)| {
                let limit = managed.app.max_memory_bytes()?;
                (managed.info.status.is_running() && managed.info.memory_bytes > limit)
                    .then_some(*pm_id)
            })
            .collect();

        for pm_id in targets {
            tracing::info!(pm_id, "restart due to memory limit");
            if let Err(err) = self.restart_id(pm_id).await {
                tracing::warn!(pm_id, error = %err, "memory restart failed");
            }
        }
        Ok(())
    }

    /// Fires cron-scheduled restarts.
    ///
    /// Mirrors `pm2/lib/Worker.js` `registerCron`: each app with a non-empty
    /// `cron_restart` schedule is restarted when wall-clock time crosses its
    /// next-fire instant, then the next fire is recomputed.
    pub async fn run_cron_restarts(&mut self) -> Result<()> {
        let now = Utc::now();
        let mut to_fire: Vec<ProcessId> = Vec::new();

        for (pm_id, managed) in &self.processes {
            let Some(spec) = managed.app.cron_restart.as_deref() else {
                continue;
            };
            if spec.is_empty() || spec == "0" {
                continue;
            }
            let schedule = match Schedule::from_str(spec) {
                Ok(schedule) => schedule,
                Err(err) => {
                    tracing::warn!(pm_id, error = %err, "invalid cron_restart spec");
                    continue;
                }
            };
            match self.cron_next.get(pm_id).copied() {
                Some(when) if when <= now => to_fire.push(*pm_id),
                None => {
                    if let Some(when) = schedule.upcoming(Utc).next() {
                        self.cron_next.insert(*pm_id, when);
                    }
                }
                _ => {}
            }
        }

        for pm_id in to_fire {
            let next_fire = self
                .processes
                .get(&pm_id)
                .and_then(|m| m.app.cron_restart.clone())
                .and_then(|spec| Schedule::from_str(&spec).ok())
                .and_then(|schedule| schedule.upcoming(Utc).next());
            if let Some(when) = next_fire {
                self.cron_next.insert(pm_id, when);
            }
            tracing::info!(pm_id, "restart due to cron schedule");
            if let Err(err) = self.restart_id(pm_id).await {
                tracing::warn!(pm_id, error = %err, "cron restart failed");
            }
        }
        Ok(())
    }

    /// Reads log tail lines for matching processes.
    pub async fn logs(&mut self, selector: Option<&Selector>, lines: usize) -> Result<Vec<String>> {
        self.refresh_and_autorestart().await?;
        let ids = match selector {
            Some(selector) => self.select_ids(selector)?,
            None => self.processes.keys().copied().collect(),
        };
        let mut output = Vec::new();

        for pm_id in ids {
            if let Some(managed) = self.processes.get(&pm_id) {
                if let Some(path) = managed.info.out_file.as_ref() {
                    output.extend(prefix_lines(
                        &managed.info.name,
                        "out",
                        tail_file(path, lines).await?,
                    ));
                }
                if let Some(path) = managed.info.error_file.as_ref() {
                    output.extend(prefix_lines(
                        &managed.info.name,
                        "err",
                        tail_file(path, lines).await?,
                    ));
                }
            }
        }

        Ok(output)
    }

    /// Loads the dump file (if any) and starts every persisted app.
    /// Returns the freshly created process list. Idempotent: apps that are
    /// already running are skipped to avoid duplicates.
    pub async fn resurrect(&mut self) -> Result<Vec<ProcessInfo>> {
        let path = self.home.dump_file();
        let bytes = match tokio::fs::read(&path).await {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Vec::new());
            }
            Err(err) => return Err(err.into()),
        };
        let apps: Vec<AppConfig> = serde_json::from_slice(&bytes)
            .map_err(|err| RspmError::Config(format!("invalid dump file: {err}")))?;

        let mut started = Vec::new();
        for app in apps {
            if self.processes.values().any(|m| m.app.name == app.name) {
                tracing::info!(name = %app.name, "skipping already-running app during resurrect");
                continue;
            }
            match self.start_app(app).await {
                Ok(mut infos) => started.append(&mut infos),
                Err(err) => tracing::warn!(error = %err, "resurrect start_app failed"),
            }
        }
        Ok(started)
    }

    /// Saves current app definitions to the dump file.
    pub async fn save(&self) -> Result<usize> {
        if self.home.dump_file().exists() {
            std::fs::copy(self.home.dump_file(), self.home.dump_backup_file())?;
        }

        let apps = self
            .processes
            .values()
            .map(|managed| managed.app.clone())
            .collect::<Vec<_>>();
        let json = serde_json::to_string_pretty(&apps)?;
        tokio::fs::write(self.home.dump_file(), json).await?;
        Ok(apps.len())
    }

    /// Sends a Unix signal to matching live processes.
    pub async fn send_signal(
        &mut self,
        selector: &Selector,
        signal_name: &str,
    ) -> Result<Vec<ProcessInfo>> {
        let signal = parse_signal(signal_name)?;
        let ids = self.select_ids(selector)?;

        for pm_id in ids {
            if let Some(pid) = self
                .processes
                .get(&pm_id)
                .and_then(|managed| managed.info.pid)
            {
                send_signal_to_pid(pid, signal)?;
            }
        }

        self.list().await
    }

    async fn spawn_for_id(&mut self, pm_id: ProcessId) -> Result<ProcessInfo> {
        let (app, instance_index) = {
            let managed = self
                .processes
                .get_mut(&pm_id)
                .ok_or_else(|| RspmError::NotFound(format!("process id {pm_id}")))?;
            managed.info.status = ProcessStatus::Launching;
            (managed.app.clone(), managed.instance_index)
        };

        let spawn = spawn_child(&self.home, pm_id, instance_index, &app).await;
        match spawn {
            Ok(mut spawned) => {
                let bus = self.bus.clone();
                let name = app.name.clone();
                // Hand the captured stdout/stderr off to dedicated tasks so
                // line-oriented output reaches both the rotating LogWriter and
                // any pub.sock subscribers (`rspm logs --follow`).
                let stdout_handle = spawn_log_forwarder(
                    pm_id,
                    name.clone(),
                    LogStream::Out,
                    spawned.stdout.take(),
                    Arc::clone(&spawned.out_writer),
                    bus.clone(),
                );
                let stderr_handle = spawn_log_forwarder(
                    pm_id,
                    name.clone(),
                    LogStream::Err,
                    spawned.stderr.take(),
                    Arc::clone(&spawned.err_writer),
                    bus.clone(),
                );

                let managed = self
                    .processes
                    .get_mut(&pm_id)
                    .ok_or_else(|| RspmError::NotFound(format!("process id {pm_id}")))?;
                managed.child = Some(spawned.child);
                managed.info.pid = Some(spawned.pid);
                managed.info.status = ProcessStatus::Online;
                managed.info.pm_uptime = Some(Utc::now());
                managed.info.out_file = Some(spawned.out_file);
                managed.info.error_file = Some(spawned.error_file);
                managed.log_tasks.clear();
                if let Some(handle) = stdout_handle {
                    managed.log_tasks.push(handle);
                }
                if let Some(handle) = stderr_handle {
                    managed.log_tasks.push(handle);
                }
                let info_snapshot = managed.info.clone();
                self.bus.publish(Event::ProcessOnline {
                    process: info_snapshot.clone(),
                });
                Ok(info_snapshot)
            }
            Err(err) => {
                if let Some(managed) = self.processes.get_mut(&pm_id) {
                    managed.info.status = ProcessStatus::Errored;
                    managed.info.pid = None;
                    managed.child = None;
                }
                Err(err)
            }
        }
    }

    async fn stop_id(&mut self, pm_id: ProcessId) -> Result<ProcessInfo> {
        let (mut child, app, name) = {
            let managed = self
                .processes
                .get_mut(&pm_id)
                .ok_or_else(|| RspmError::NotFound(format!("process id {pm_id}")))?;
            managed.info.status = ProcessStatus::Stopping;
            managed.abort_watcher();
            managed.abort_log_tasks();
            (
                managed.child.take(),
                managed.app.clone(),
                managed.info.name.clone(),
            )
        };

        let mut exit_code: Option<i32> = None;
        if let Some(child) = child.as_mut() {
            if let Some(pid) = child.id() {
                let _ = send_signal_to_pid(pid, Signal::SIGINT);
            }

            match timeout(Duration::from_millis(app.kill_timeout_ms), child.wait()).await {
                Ok(wait_result) => {
                    exit_code = wait_result?.code();
                }
                Err(_) => {
                    child.kill().await?;
                    exit_code = child.wait().await.ok().and_then(|status| status.code());
                }
            }
        }

        let pid_path = self.home.app_pid_file(&name, pm_id);
        let _ = std::fs::remove_file(pid_path);
        let managed = self
            .processes
            .get_mut(&pm_id)
            .ok_or_else(|| RspmError::NotFound(format!("process id {pm_id}")))?;
        managed.info.status = ProcessStatus::Stopped;
        managed.info.pid = None;
        managed.info.pm_uptime = None;
        managed.child = None;
        self.bus.publish(Event::ProcessExit {
            pm_id,
            code: exit_code,
        });
        Ok(managed.info.clone())
    }

    async fn restart_id(&mut self, pm_id: ProcessId) -> Result<ProcessInfo> {
        self.stop_id(pm_id).await?;
        if let Some(managed) = self.processes.get_mut(&pm_id) {
            managed.info.restart_time = managed.info.restart_time.saturating_add(1);
        }
        let info = self.spawn_for_id(pm_id).await?;
        self.ensure_watcher(pm_id);
        Ok(info)
    }

    async fn refresh_and_autorestart(&mut self) -> Result<()> {
        let restart_ids = self.refresh_statuses();
        let now = tokio::time::Instant::now();
        for pm_id in restart_ids {
            // Honor `restart_delay` / exp-backoff: skip until the scheduled
            // restart time arrives. Worker tick + future refreshes will pick
            // it up once the deadline lapses.
            let due = self
                .processes
                .get(&pm_id)
                .and_then(|managed| managed.next_restart_at)
                .map(|deadline| deadline <= now)
                .unwrap_or(true);
            if !due {
                continue;
            }

            if let Some(managed) = self.processes.get_mut(&pm_id) {
                managed.info.restart_time = managed.info.restart_time.saturating_add(1);
                managed.next_restart_at = None;
                managed.abort_watcher();
            }
            if let Err(err) = self.spawn_for_id(pm_id).await {
                tracing::warn!(pm_id, error = %err, "auto restart failed");
            } else {
                self.ensure_watcher(pm_id);
            }
        }
        Ok(())
    }

    /// Polls each child for completion, updates status + restart bookkeeping,
    /// and returns the ids that should be respawned by
    /// [`Self::refresh_and_autorestart`]. Mirrors the unstable_restarts /
    /// max_restarts logic of `pm2/lib/God.js:handleExit`.
    fn refresh_statuses(&mut self) -> Vec<ProcessId> {
        let mut restart_ids = Vec::new();
        let mut exited: Vec<(ProcessId, Option<i32>)> = Vec::new();
        let now_instant = tokio::time::Instant::now();

        for (pm_id, managed) in &mut self.processes {
            let exit_status = match managed.child.as_mut() {
                Some(child) => match child.try_wait() {
                    Ok(status) => status,
                    Err(err) => {
                        tracing::warn!(pm_id, error = %err, "failed to poll child");
                        None
                    }
                },
                None => None,
            };

            let Some(status) = exit_status else {
                continue;
            };

            exited.push((*pm_id, status.code()));
            managed.abort_log_tasks();
            managed.child = None;
            managed.info.pid = None;
            let uptime_ms = managed
                .info
                .pm_uptime
                .map(|t| (Utc::now() - t).num_milliseconds().max(0) as u64)
                .unwrap_or(0);
            managed.info.pm_uptime = None;

            // Exit-code based status. `success()` covers code 0 OR the
            // user listed it in `stop_exit_codes` — both are treated as
            // intentional and DO NOT trigger restart.
            let intentional = status.success()
                || status
                    .code()
                    .map(|code| managed.app.stop_exit_codes.contains(&code))
                    .unwrap_or(false);

            managed.info.status = if intentional {
                ProcessStatus::Stopped
            } else {
                ProcessStatus::Errored
            };

            if intentional || !managed.app.auto_restart {
                continue;
            }

            // Track crash-loop: short-lived exits bump unstable_restarts.
            if uptime_ms < managed.app.min_uptime_ms {
                managed.info.unstable_restarts = managed.info.unstable_restarts.saturating_add(1);
            }

            if managed.info.unstable_restarts >= managed.app.max_restarts
                || managed.info.restart_time >= managed.app.max_restarts
            {
                managed.info.status = ProcessStatus::Errored;
                tracing::warn!(
                    pm_id,
                    unstable_restarts = managed.info.unstable_restarts,
                    restart_time = managed.info.restart_time,
                    "max_restarts exceeded, marking errored"
                );
                continue;
            }

            // Compute delay before next restart.
            let delay_ms = match managed.app.exp_backoff_restart_delay_ms {
                Some(base) if base > 0 => {
                    let next =
                        rspm_core::defaults::next_exp_backoff(managed.prev_restart_delay_ms, base);
                    managed.prev_restart_delay_ms = next;
                    next
                }
                _ => managed.app.restart_delay_ms,
            };

            managed.next_restart_at = if delay_ms == 0 {
                None
            } else {
                Some(now_instant + Duration::from_millis(delay_ms))
            };
            managed.info.status = ProcessStatus::Waiting;
            restart_ids.push(*pm_id);
        }

        for (pm_id, code) in exited {
            self.bus.publish(Event::ProcessExit { pm_id, code });
        }

        restart_ids
    }

    /// Resets `prev_restart_delay_ms` for processes that have been stable
    /// long enough. Mirrors PM2 worker tick behavior: when an app stays up
    /// past [`rspm_core::defaults::EXP_BACKOFF_RESET_TIMER_MS`], the next
    /// crash should restart with the base delay instead of the escalated
    /// value from the previous crash loop.
    fn reset_stable_backoff(&mut self) {
        let now = Utc::now();
        let reset_threshold =
            chrono::Duration::milliseconds(rspm_core::defaults::EXP_BACKOFF_RESET_TIMER_MS as i64);
        for managed in self.processes.values_mut() {
            if managed.prev_restart_delay_ms == 0 {
                continue;
            }
            let Some(uptime_start) = managed.info.pm_uptime else {
                continue;
            };
            if managed.info.status == ProcessStatus::Online
                && (now - uptime_start) > reset_threshold
            {
                managed.prev_restart_delay_ms = 0;
            }
        }
    }

    fn select_ids(&self, selector: &Selector) -> Result<Vec<ProcessId>> {
        let ids = match selector {
            Selector::All => self.processes.keys().copied().collect::<Vec<_>>(),
            Selector::Id(id) if self.processes.contains_key(id) => vec![*id],
            Selector::Id(id) => return Err(RspmError::NotFound(format!("process id {id}"))),
            Selector::Name(name) => self
                .processes
                .iter()
                .filter_map(|(pm_id, managed)| (managed.info.name == *name).then_some(*pm_id))
                .collect::<Vec<_>>(),
        };

        if ids.is_empty() {
            return Err(RspmError::NotFound(format!("selector {selector:?}")));
        }

        Ok(ids)
    }

    fn allocate_id(&mut self) -> ProcessId {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        id
    }
}

struct SpawnedChild {
    child: Child,
    pid: u32,
    out_file: PathBuf,
    error_file: PathBuf,
    stdout: Option<ChildStdout>,
    stderr: Option<ChildStderr>,
    out_writer: Arc<StdMutex<LogWriter>>,
    err_writer: Arc<StdMutex<LogWriter>>,
}

fn build_log_opts(app: &AppConfig) -> LogOpts {
    LogOpts {
        prefix_timestamp: app.prefix_timestamp,
        log_date_format: app.log_date_format.clone(),
        merge_label: if app.merge_logs {
            Some(app.name.clone())
        } else {
            None
        },
        max_bytes: Some(rspm_core::defaults::log_max_bytes()),
        max_archives: rspm_core::defaults::log_max_archives(),
    }
}

async fn spawn_child(
    home: &RspmHome,
    pm_id: ProcessId,
    instance_index: u32,
    app: &AppConfig,
) -> Result<SpawnedChild> {
    let out_file = app
        .combined_file
        .clone()
        .or_else(|| app.out_file.clone())
        .unwrap_or_else(|| home.app_log_file(&app.name, "out"));
    let error_file = app
        .combined_file
        .clone()
        .or_else(|| app.error_file.clone())
        .unwrap_or_else(|| home.app_log_file(&app.name, "error"));

    let out_writer = Arc::new(StdMutex::new(LogWriter::new(
        &out_file,
        build_log_opts(app),
    )?));
    let err_writer = if out_file == error_file {
        Arc::clone(&out_writer)
    } else {
        Arc::new(StdMutex::new(LogWriter::new(
            &error_file,
            build_log_opts(app),
        )?))
    };

    let mut command = build_command(app);
    if let Some(cwd) = app.cwd.as_ref() {
        command.current_dir(cwd);
    }
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    command.stdin(Stdio::null());
    command.kill_on_drop(false);
    command.envs(app.env.iter());
    command.env("pm_id", pm_id.to_string());
    command.env("name", &app.name);
    command.env(&app.instance_var, instance_index.to_string());
    apply_cluster_env(&mut command, app, instance_index);
    if app.wait_ready {
        let ready_file = ready_file_path(home, &app.name, pm_id);
        let _ = std::fs::remove_file(&ready_file);
        if let Some(value) = ready_file.to_str() {
            command.env("RSPM_READY_FILE", value);
        }
    }

    let mut child = command.spawn()?;
    let pid = child
        .id()
        .ok_or_else(|| RspmError::Daemon("spawned child did not expose a pid".to_owned()))?;
    write_app_pid(home, &app.name, pm_id, pid)?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    Ok(SpawnedChild {
        child,
        pid,
        out_file,
        error_file,
        stdout,
        stderr,
        out_writer,
        err_writer,
    })
}

/// Reads the child's stdout/stderr line by line, writes each line through the
/// rotating [`LogWriter`], and publishes an [`Event::Log`] so subscribers on
/// `pub.sock` can stream logs in real time.
fn spawn_log_forwarder(
    pm_id: ProcessId,
    name: String,
    stream: LogStream,
    source: Option<impl tokio::io::AsyncRead + Send + Unpin + 'static>,
    writer: Arc<StdMutex<LogWriter>>,
    bus: PubSubBus,
) -> Option<JoinHandle<()>> {
    let source = source?;
    Some(tokio::spawn(async move {
        let mut reader = BufReader::new(source).lines();
        loop {
            match reader.next_line().await {
                Ok(Some(line)) => {
                    if let Ok(mut writer) = writer.lock()
                        && let Err(err) = writer.write_line(line.as_bytes())
                    {
                        tracing::warn!(pm_id, error = %err, "log write failed");
                    }
                    bus.publish(Event::Log {
                        pm_id,
                        name: name.clone(),
                        stream: stream.clone(),
                        data: line,
                        at: Utc::now(),
                    });
                }
                Ok(None) => break,
                Err(err) => {
                    tracing::debug!(pm_id, error = %err, "log read ended");
                    break;
                }
            }
        }
    }))
}

fn ready_file_path(home: &RspmHome, name: &str, pm_id: ProcessId) -> PathBuf {
    home.pid_dir().join(format!(
        "{}-{pm_id}.ready",
        rspm_core::paths::sanitize_name(name)
    ))
}

fn apply_cluster_env(command: &mut Command, app: &AppConfig, instance_index: u32) {
    command.env("RSPM_INSTANCE_ID", instance_index.to_string());
    command.env("RSPM_EXEC_MODE", app.execution_mode.as_pm2_str());
    command.env("RSPM_INSTANCES", app.instances.resolve().to_string());
    if matches!(app.execution_mode, ExecutionMode::ClusterMode) {
        // Tell the app it should bind with SO_REUSEPORT instead of relying
        // on a passed fd (no Node.js cluster module on our side).
        command.env("RSPM_CLUSTER", "1");
    }
}

fn build_command(app: &AppConfig) -> Command {
    let interpreter = app.interpreter.as_ref().and_then(|path| {
        if path.as_os_str() == "none" {
            None
        } else {
            Some(path)
        }
    });

    if let Some(interpreter) = interpreter {
        let mut command = Command::new(interpreter);
        command.args(&app.interpreter_args);
        command.arg(&app.script);
        command.args(&app.args);
        return command;
    }

    if is_node_script(&app.script) {
        let mut command = Command::new("node");
        command.arg(&app.script);
        command.args(&app.args);
        return command;
    }

    if app.script.extension().and_then(|value| value.to_str()) == Some("py") {
        let mut command = Command::new("python3");
        command.arg(&app.script);
        command.args(&app.args);
        return command;
    }

    let mut command = Command::new(&app.script);
    command.args(&app.args);
    command
}

fn is_node_script(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|value| value.to_str()),
        Some("js" | "mjs" | "cjs")
    )
}

fn write_app_pid(home: &RspmHome, name: &str, pm_id: ProcessId, pid: u32) -> Result<()> {
    let path = home.app_pid_file(name, pm_id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, pid.to_string())?;
    Ok(())
}

fn send_signal_to_pid(pid: u32, signal: Signal) -> Result<()> {
    let raw_pid = i32::try_from(pid)
        .map_err(|_| RspmError::Signal(format!("pid {pid} does not fit platform pid")))?;
    kill(Pid::from_raw(raw_pid), signal).map_err(|err| RspmError::Signal(err.to_string()))
}

fn parse_signal(signal: &str) -> Result<Signal> {
    match signal.trim_start_matches("SIG") {
        "INT" => Ok(Signal::SIGINT),
        "TERM" => Ok(Signal::SIGTERM),
        "KILL" => Ok(Signal::SIGKILL),
        "HUP" => Ok(Signal::SIGHUP),
        "USR1" => Ok(Signal::SIGUSR1),
        "USR2" => Ok(Signal::SIGUSR2),
        other => Err(RspmError::Signal(format!("unsupported signal {other}"))),
    }
}

fn prefix_lines(name: &str, stream: &str, lines: Vec<String>) -> Vec<String> {
    lines
        .into_iter()
        .map(|line| format!("[{name}] [{stream}] {line}"))
        .collect()
}

fn watch_enabled(spec: &WatchSpec) -> bool {
    match spec {
        WatchSpec::Enabled(value) => *value,
        WatchSpec::Paths(paths) => !paths.is_empty(),
    }
}

fn watcher_root(app: &AppConfig) -> PathBuf {
    if let Some(cwd) = app.cwd.as_ref() {
        return cwd.clone();
    }
    if let Some(parent) = app
        .script
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        return parent.to_path_buf();
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

async fn run_watcher(
    mut watcher: AppWatcher,
    pm_id: ProcessId,
    tx: mpsc::UnboundedSender<ProcessId>,
) {
    while let Some(event) = watcher.next_event().await {
        tracing::debug!(pm_id, path = %event.path.display(), "watch event");
        if tx.send(pm_id).is_err() {
            break;
        }
    }
}

/// Waits for a freshly-spawned child to declare itself ready before the
/// caller (soft reload) proceeds. When `wait_ready` is set the daemon polls
/// for `$RSPM_READY_FILE` to be created (the child creates it once warmed
/// up) for up to `listen_timeout_ms`. When `wait_ready` is unset we sleep a
/// brief window so the new child can finish binding its listening socket.
async fn wait_for_ready(home: &RspmHome, app: &AppConfig, pm_id: ProcessId) {
    if !app.wait_ready {
        tokio::time::sleep(Duration::from_millis(100)).await;
        return;
    }

    let ready_path = ready_file_path(home, &app.name, pm_id);
    let deadline = tokio::time::Instant::now() + Duration::from_millis(app.listen_timeout_ms);
    let poll = Duration::from_millis(50);
    while tokio::time::Instant::now() < deadline {
        if tokio::fs::metadata(&ready_path).await.is_ok() {
            let _ = tokio::fs::remove_file(&ready_path).await;
            return;
        }
        tokio::time::sleep(poll).await;
    }
    tracing::warn!(
        pm_id,
        path = %ready_path.display(),
        "wait_ready timed out without ready sentinel"
    );
}

#[allow(dead_code)]
fn _path_env(path: &Path) -> Result<&str> {
    path_to_string(path)
}
