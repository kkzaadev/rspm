# RSPM (Rust PM2) — Plan Pembangunan Ulang PM2 dalam Rust

> Target file plan ini dieksekusi oleh **AI vibe coding** milik user, bukan oleh Claude yang sedang menyusun plan.
> Plan ini bersifat **production-ready blueprint**: ada strategi file, breakdown fungsi, vibe coding rules, dan ~120 task atomik dengan referensi PM2.

---

## 1. Context

### 1.1 Tujuan

Membangun **`rspm`** (Rust Process Manager) sebagai **functional-equivalent** dari PM2 (Node.js) namun ditulis dalam Rust. Versi awal **`0.0.1`**, dirilis sebagai **open source** dengan kualitas production.

### 1.2 Kenapa Perlu Plan Selengkap Ini

- AI vibe coding user butuh **task atomik** + **referensi file PM2** + **flow pseudo-code** supaya tidak melenceng dari behavior PM2.
- PM2 punya ratusan edge case (signal handling, soft reload, watch ignore, log rotation) yang harus direplikasi 1:1.
- Tanpa plan ini, kode Rust hasil generate akan jadi "PM2 yang mirip-mirip" — bukan drop-in replacement.

### 1.3 Outcome yang Diharapkan

- Binary tunggal `rspm` (release build) yang bisa: `start`, `stop`, `restart`, `reload`, `delete`, `list`, `logs`, `monit`, `save`, `resurrect`, `startup`, `dump`, `describe`, `prettylist`, `jlist`, `kill`, `update`, `flush`, `reloadLogs`, `ping`, `sendSignal`, `scale`, `deploy`, `module:install`, dll.
- 100% kompatibel dengan **`ecosystem.config.js`** PM2 (parsing via embedded JS engine).
- Kompatibel dengan format `pm2 startup` (systemd/openrc/sysv unit file).
- Cluster mode lintas-bahasa via **`SO_REUSEPORT`** (bukan Node.js cluster).

---

## 2. Decisions (Sudah Dikonfirmasi User)

| ID  | Keputusan       | Nilai                                                        |
| --- | --------------- | ------------------------------------------------------------ |
| D1  | Scope v0.0.1    | Full feature parity dengan PM2                               |
| D2  | Config formats  | TOML + YAML + JSON + `ecosystem.config.js` (via embedded JS) |
| D3  | Wire protocol   | Pure Rust protocol baru (BUKAN kompatibel `pm2-axon`)        |
| D4  | Cluster mode    | Implement `SO_REUSEPORT` (universal untuk semua bahasa)      |
| D5  | Platform v0.0.1 | **Linux only** (macOS/Windows menyusul)                      |
| D6  | Workspace       | Multi-crate cargo workspace                                  |
| D7  | Storage         | JSON file (compatible-style dengan PM2 `dump.pm2`)           |
| D8  | Home directory  | `$RSPM_HOME` env var, default `~/.rspm/`                     |

---

## 3. Roadmap Versioning

| Versi     | Milestone                                                                  |
| --------- | -------------------------------------------------------------------------- |
| **0.0.1** | Core daemon + IPC + CLI minimum (start, stop, list, logs, restart, delete) |
| 0.1.0     | Cluster mode + watcher + log rotation + ecosystem.config                   |
| 0.2.0     | Monitoring (CPU/MEM), reload (zero downtime), describe, jlist              |
| 0.3.0     | Startup script generator + save/resurrect                                  |
| 0.4.0     | HTTP API + TUI dashboard (monit)                                           |
| 0.5.0     | Modules + deploy (SSH)                                                     |
| 1.0.0     | Stable, dokumentasi lengkap, CI/CD release                                 |

---

## 4. Workspace Structure

### 4.1 Layout Direktori

```
rspm/
├── Cargo.toml                    # workspace root
├── CLAUDE.md                     # instruction untuk vibe coding (Section 5)
├── README.md
├── LICENSE                       # MIT (sama dengan PM2)
├── .gitignore
├── rust-toolchain.toml           # pin Rust version
├── deny.toml                     # cargo-deny config (security audit)
├── pm2/                          # READ-ONLY reference PM2 source
├── crates/
│   ├── rspm-core/                # types, error, constants, paths
│   ├── rspm-protocol/            # wire protocol (Request/Response/Event)
│   ├── rspm-config/              # parsing TOML/YAML/JSON/JS ecosystem
│   ├── rspm-ipc/                 # IPC transport (Unix Domain Socket)
│   ├── rspm-daemon/              # God daemon (supervisor process)
│   ├── rspm-client/              # client lib (CLI use this)
│   ├── rspm-cli/                 # binary entrypoint
│   ├── rspm-watcher/             # file watcher (notify crate)
│   ├── rspm-monitor/             # CPU/MEM sampler (sysinfo)
│   ├── rspm-logs/                # log file writer + rotation
│   ├── rspm-cluster/             # SO_REUSEPORT helper
│   ├── rspm-modules/             # module install/uninstall
│   ├── rspm-startup/             # systemd/openrc/sysv generator
│   ├── rspm-deploy/              # SSH deploy (russh)
│   ├── rspm-dashboard/           # TUI (ratatui) — pm2 monit
│   └── rspm-http/                # optional HTTP API (axum)
├── docs/
│   ├── architecture.md
│   ├── protocol.md
│   ├── cli-reference.md
│   └── migration-from-pm2.md
├── tests/                        # end-to-end test
└── examples/
    ├── ecosystem.config.js
    ├── apps.toml
    └── cluster-node.js
```

### 4.2 Dependency Graph antar Crate

```
rspm-cli ──► rspm-client ──► rspm-protocol ──► rspm-core
                │                  ▲
                └──► rspm-ipc ─────┘
                       ▲
rspm-daemon ───────────┤
   ├── rspm-config ────┤
   ├── rspm-watcher ───┤
   ├── rspm-monitor ───┤
   ├── rspm-logs ──────┤
   ├── rspm-cluster ───┤
   ├── rspm-startup ───┤
   ├── rspm-modules ───┤
   ├── rspm-deploy ────┤
   └── rspm-http ──────┘
                       ▲
rspm-dashboard ────────┘  (TUI via client lib)
```

**Aturan keras**: tidak boleh ada **cyclic dependency**. `rspm-core` tidak boleh import crate lain di workspace.

---

## 5. Konten CLAUDE.md (Project Root)

> File ini ditulis di `/home/zdanysfa/rspm/CLAUDE.md` (override yang sudah ada).
> Vibe coding AI akan membaca ini setiap session.

```markdown
# CLAUDE.md — RSPM (Rust PM2)

## Tujuan

Membangun ulang PM2 (Process Manager 2) dalam Rust dengan **functional parity penuh**.
Nama project: `rspm`. Versi awal: `0.0.1`. Lisensi: MIT.

## Referensi Source Mutlak

Sebelum menulis kode untuk task apa pun, **WAJIB baca file PM2 referensi** yang tercantum di plan task.
Folder `pm2/` adalah **READ-ONLY**. Jangan modifikasi.

## Bahasa & Tooling

- Rust edition 2024 (pin via `rust-toolchain.toml`)
- Async runtime: `tokio` (multi-thread)
- CLI parsing: `clap` v4 (derive macros)
- Serialisasi: `serde` + `serde_json` + `serde_yaml` + `toml`
- Embedded JS (untuk ecosystem.config.js): `boa_engine`
- File watcher: `notify`
- System info: `sysinfo`
- Unix syscall: `nix`
- Error: `thiserror` (library), `anyhow` (binary)
- Logging: `tracing` + `tracing-subscriber`
- TUI: `ratatui` + `crossterm`
- HTTP: `axum` (optional feature)
- SSH: `russh`
- Templating: `tera`

## Workspace

Multi-crate workspace. Lihat plan section 4.

## Aturan Vibe Coding (Hard Rules)

R1. Selalu **TDD**: tulis test dulu, baru implementasi.
R2. Jangan pernah `unwrap()` di kode produksi — pakai `?` + `thiserror`.
R3. Jangan tambah dependency baru tanpa update `deny.toml` dan tanpa justifikasi di commit message.
R4. Setiap fungsi public harus punya doc comment `///` + minimal 1 contoh.
R5. Setiap PR hanya menyelesaikan **1 task atomik** dari plan (T-prefix).
R6. Field name di protocol/config harus persis sama dengan PM2 (snake_case), kecuali dicatat di plan.
R7. Kalau ragu antara "ikuti PM2" vs "lebih bersih ala Rust", **selalu ikuti PM2** untuk v0.0.1.
R8. Tidak ada `panic!`, `process::exit` di library crate. Hanya `rspm-cli` dan `rspm-daemon` (entry point) yang boleh.
R9. Semua path file pakai `PathBuf`, jangan `String`. Semua waktu pakai `chrono::DateTime<Utc>`.
R10. Default value harus didefinisikan di `rspm-core::defaults` — tidak boleh hardcode di tempat lain.

## Direktori PM2 Home (Compat)
```

$RSPM_HOME (default ~/.rspm/)
├── pids/
├── logs/
├── modules/
├── pm2.log # daemon log (nama dipertahankan untuk kompat)
├── pm2.pid # daemon pid
├── rpc.sock # IPC request/reply
├── pub.sock # IPC pub/sub events
└── dump.rspm # persisted process list

```

## Konvensi Naming Internal
- `App` = definisi aplikasi (dari config).
- `Process` = instance running (1 app cluster mode = N process).
- `ProcessId` (`pm_id` di PM2) = monotonic counter dari daemon.
- `Bus` = pub/sub event channel.
- `God` = daemon supervisor (nama dipertahankan dari PM2).

## Workflow per Task
1. Baca task dari plan (T-prefix).
2. Baca file PM2 referensi yang disebut task.
3. Tulis test di crate yang sama.
4. Implementasi minimal supaya test hijau.
5. Jalankan `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`.
6. Commit dengan format: `feat(crate): T0.5 ringkasan singkat`.
```

---

## 6. Vibe Coding Rules (Detail R1–R10)

| Rule                   | Detail                                                                                                           | Konsekuensi Pelanggaran            |
| ---------------------- | ---------------------------------------------------------------------------------------------------------------- | ---------------------------------- |
| **R1 TDD**             | `cargo test` sebelum implementasi. Test letakkan di `tests/` (integration) atau `#[cfg(test)] mod tests` (unit). | PR ditolak.                        |
| **R2 No unwrap**       | Pakai `?` propagation. Untuk `Option::None` yang impossible, pakai `.expect("invariant: ...")` dengan reason.    | Clippy lint `unwrap_used` di-deny. |
| **R3 Deps locked**     | Update `deny.toml` & `Cargo.lock` bersama. Jelaskan kenapa di commit body.                                       | `cargo deny check` gagal di CI.    |
| **R4 Docs**            | `cargo doc --no-deps` harus 0 warning. Setiap `pub fn` dokumentasi.                                              | CI gagal.                          |
| **R5 One task per PR** | Branch name: `task/T1.3-daemon-bootstrap`.                                                                       | Reviewer minta split.              |
| **R6 Field parity**    | Bandingkan dengan `pm2/lib/API/schema.json` saat tambah field config.                                            | Reviewer reject.                   |
| **R7 Ikuti PM2**       | Behavior ambigu? Cek dulu source PM2, replikasi. Catat divergence di `docs/migration-from-pm2.md`.               | Bug parity.                        |
| **R8 No panic in lib** | Lint `clippy::panic` di-deny untuk crate non-binary.                                                             | CI gagal.                          |
| **R9 Types**           | `PathBuf`, `DateTime<Utc>`, `Duration`, `u32 pm_id`.                                                             | Review reject.                     |
| **R10 Defaults**       | `rspm_core::defaults::KILL_TIMEOUT`, `WORKER_INTERVAL`, dll.                                                     | Hardcoded magic number reject.     |

---

## 7. File Strategy & Function Breakdown (Per Crate)

### 7.1 `rspm-core` — Foundation Types

**Tujuan**: Tipe data dasar yang dipakai semua crate. **Tidak boleh import crate workspace lain**.

```
crates/rspm-core/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── error.rs           # RspmError (thiserror)
    ├── constants.rs       # KILL_TIMEOUT, WORKER_INTERVAL, DEFAULT_*
    ├── defaults.rs        # default-fn untuk serde
    ├── paths.rs           # RspmHome struct + resolve()
    ├── types/
    │   ├── mod.rs
    │   ├── app.rs         # AppConfig, ExecutionMode enum
    │   ├── process.rs     # ProcessInfo, ProcessStatus enum
    │   ├── metric.rs      # CpuSample, MemSample
    │   └── env.rs         # EnvMap (BTreeMap<String,String>)
    └── version.rs         # pkg_version, build_info
```

**Fungsi utama**:

- `RspmHome::from_env() -> Result<Self>` — baca `$RSPM_HOME` atau default.
- `RspmHome::rpc_socket(&self) -> PathBuf`
- `RspmHome::pub_socket(&self) -> PathBuf`
- `RspmHome::dump_file(&self) -> PathBuf`
- `RspmHome::log_dir(&self) -> PathBuf`
- `RspmHome::pid_dir(&self) -> PathBuf`
- `ProcessStatus` enum: `Online`, `Stopping`, `Stopped`, `Errored`, `OneLaunchStatus`, `Launching`, `Waiting`.

**PM2 Refs**: `pm2/constants.js`, `pm2/paths.js`.

---

### 7.2 `rspm-protocol` — Wire Protocol

**Tujuan**: Definisi message Request/Response/Event antara client ↔ daemon.

```
crates/rspm-protocol/
└── src/
    ├── lib.rs
    ├── frame.rs           # length-prefix framing (u32 BE + JSON payload)
    ├── request.rs         # enum Request { Start, Stop, Restart, List, ... }
    ├── response.rs        # enum Response { Ack, ProcessList, Error, ... }
    ├── event.rs           # enum Event { ProcessOnline, ProcessExit, Log, ... }
    └── version.rs         # const PROTOCOL_VERSION: u32 = 1
```

**Format frame**:

```
[4 byte u32 BE length] [N byte JSON serde::Serialize body]
```

**Method list (parity dengan PM2 RPC method)**:

- `prepareJson`, `startProcessId`, `stopProcessId`, `restartProcessId`
- `deleteProcessId`, `reloadProcessId`, `getMonitorData`, `getSystemData`
- `findByName`, `msgProcess`, `sendSignalToProcessName`, `sendDataToProcessId`
- `reloadLogs`, `dumpProcessList`, `saveProcessList`, `monitor`, `unmonitor`
- `getVersion`, `notifyKillPM2`, `forceGc`, `ping`

**Event list**:

- `process:online`, `process:exit`, `process:msg`, `process:event`
- `log:out`, `log:err`, `monit:reload`, `system:warn`

**PM2 Refs**: `pm2/lib/God/ActionMethods.js` (all RPC methods), `pm2/lib/God.js:200-400` (event emits).

---

### 7.3 `rspm-config` — Configuration Parser

**Tujuan**: Parse semua format input config user.

```
crates/rspm-config/
└── src/
    ├── lib.rs
    ├── loader.rs          # load_file(path) -> AppConfig (auto-detect format)
    ├── format/
    │   ├── mod.rs
    │   ├── toml.rs
    │   ├── yaml.rs
    │   ├── json.rs
    │   └── ecosystem.rs   # ecosystem.config.js via boa_engine
    ├── normalize.rs       # apply defaults, validate fields
    ├── env_expand.rs      # ${VAR} expansion
    └── schema.rs          # Serde-mapped AppConfigInput (sebelum normalize)
```

**Fungsi utama**:

- `load_file(path: &Path) -> Result<Vec<AppConfig>>` — auto-detect extension.
- `parse_ecosystem(js_src: &str) -> Result<EcosystemFile>` — pakai `boa_engine`, evaluate `module.exports = { apps: [...] }`, baca property `apps`.
- `normalize::apply_defaults(input: AppConfigInput) -> AppConfig`.
- `env_expand::expand(value: &str, env: &EnvMap) -> String`.

**PM2 Refs**: `pm2/lib/Common.js:105-572` (`prepareAppConf`), `pm2/lib/API/schema.json` (canonical field list).

**Field mapping (subset)**:
| PM2 field | Rust struct field | Type |
|-----------|-------------------|------|
| `name` | `name` | `String` |
| `script` | `script` | `PathBuf` |
| `args` | `args` | `Vec<String>` |
| `cwd` | `cwd` | `Option<PathBuf>` |
| `exec_mode` | `execution_mode` | `enum {Fork, Cluster}` |
| `instances` | `instances` | `InstanceCount` (Int / `max` / `-1`) |
| `max_memory_restart` | `max_memory_restart` | `Option<ByteSize>` |
| `autorestart` | `auto_restart` | `bool` (default true) |
| `watch` | `watch` | `WatchSpec` (bool / Vec<String>) |
| `ignore_watch` | `ignore_watch` | `Vec<String>` |
| `kill_timeout` | `kill_timeout_ms` | `u64` (default 1600) |
| `min_uptime` | `min_uptime_ms` | `u64` |
| `max_restarts` | `max_restarts` | `u32` (default 16) |
| `env` | `env` | `EnvMap` |
| `env_*` | `env_overrides` | `BTreeMap<String, EnvMap>` |
| `error_file` / `out_file` / `log_file` | `error_file` / `out_file` / `combined_file` | `Option<PathBuf>` |
| `log_date_format` | `log_date_format` | `Option<String>` (strftime) |
| `merge_logs` | `merge_logs` | `bool` |
| `time` | `prefix_timestamp` | `bool` |
| `cron_restart` | `cron_restart` | `Option<String>` |
| `interpreter` | `interpreter` | `Option<PathBuf>` |
| `interpreter_args` / `node_args` | `interpreter_args` | `Vec<String>` |
| `instance_var` | `instance_var` | `Option<String>` (default `NODE_APP_INSTANCE`) |
| `wait_ready` | `wait_ready` | `bool` |
| `listen_timeout` | `listen_timeout_ms` | `u64` (default 8000) |

---

### 7.4 `rspm-ipc` — IPC Transport

**Tujuan**: Unix Domain Socket transport untuk request/reply + pub/sub.

```
crates/rspm-ipc/
└── src/
    ├── lib.rs
    ├── server.rs          # IpcServer (bind, accept loop)
    ├── client.rs          # IpcClient (connect, request)
    ├── codec.rs           # FrameCodec (tokio_util::codec)
    ├── bus.rs             # PubSubBus (tokio broadcast channel)
    └── handshake.rs       # version check (PROTOCOL_VERSION)
```

**Fungsi utama**:

- `IpcServer::bind(rpc_path: &Path, pub_path: &Path) -> Result<Self>`.
- `IpcServer::run<H: Handler>(self, handler: H)`.
- `IpcClient::connect(rpc_path: &Path) -> Result<Self>`.
- `IpcClient::call(&mut self, req: Request) -> Result<Response>`.
- `Bus::subscribe() -> BroadcastReceiver<Event>`.
- `Bus::publish(event: Event)`.

**PM2 Refs**: `pm2/lib/Daemon.js:200-300` (RPC server setup), `pm2/lib/Client.js:200-400` (client connect).

---

### 7.5 `rspm-daemon` — God Process

**Tujuan**: Long-running supervisor. Manage semua child process.

```
crates/rspm-daemon/
└── src/
    ├── lib.rs
    ├── main.rs            # binary entry (called via `rspm --daemon`)
    ├── god.rs             # God struct, singleton state
    ├── handler.rs         # dispatch Request -> handler fn
    ├── handlers/
    │   ├── mod.rs
    │   ├── start.rs       # start_app_handler
    │   ├── stop.rs
    │   ├── restart.rs
    │   ├── reload.rs      # soft reload (zero downtime)
    │   ├── delete.rs
    │   ├── list.rs
    │   ├── describe.rs
    │   ├── monit.rs       # getMonitorData
    │   ├── dump.rs        # save/resurrect
    │   └── signal.rs      # sendSignalToProcessName
    ├── supervisor/
    │   ├── mod.rs
    │   ├── fork_mode.rs   # spawn single child
    │   ├── cluster_mode.rs# spawn N children + SO_REUSEPORT
    │   └── lifecycle.rs   # on_exit, restart_with_delay
    ├── worker.rs          # background tasks (mem check, cron restart)
    ├── pid_file.rs        # daemon pidfile lock
    └── log_capture.rs     # capture child stdout/stderr -> log files
```

**Fungsi utama**:

- `God::new(home: RspmHome) -> Self`.
- `God::start_app(&mut self, app: AppConfig) -> Result<Vec<ProcessId>>`.
- `God::on_child_exit(&mut self, pm_id: ProcessId, status: ExitStatus)`.
- `God::soft_reload(&mut self, name: &str) -> Result<()>`.
- `Supervisor::fork_one(app: &AppConfig, pm_id: ProcessId) -> Result<Child>`.
- `Worker::tick(&mut god)` — dipanggil setiap `WORKER_INTERVAL` ms.

**PM2 Refs**:

- `pm2/lib/Daemon.js` (entry)
- `pm2/lib/God.js` (singleton)
- `pm2/lib/God/ActionMethods.js` (semua handler)
- `pm2/lib/God/ForkMode.js` (fork)
- `pm2/lib/God/ClusterMode.js` (cluster)
- `pm2/lib/God/Reload.js` (soft/hard reload)
- `pm2/lib/Worker.js` (background)

---

### 7.6 `rspm-client` — Client Library

**Tujuan**: Library yang dipakai CLI dan dashboard untuk berbicara ke daemon. Auto-spawn daemon kalau belum jalan.

```
crates/rspm-client/
└── src/
    ├── lib.rs
    ├── client.rs          # RspmClient struct
    ├── daemon_launcher.rs # spawn daemon kalau belum ada
    ├── reconnect.rs       # retry policy
    └── api.rs             # high-level API (start_apps, list, logs_stream, ...)
```

**Fungsi utama**:

- `RspmClient::connect_or_launch(home: &RspmHome) -> Result<Self>`.
- `RspmClient::start_app(&mut self, app: AppConfig) -> Result<Vec<ProcessInfo>>`.
- `RspmClient::list(&mut self) -> Result<Vec<ProcessInfo>>`.
- `RspmClient::stream_logs(&mut self, filter: LogFilter) -> impl Stream<Item=LogLine>`.
- `daemon_launcher::launch_if_needed(home: &RspmHome) -> Result<()>` — fork-detach pattern.

**PM2 Refs**: `pm2/lib/Client.js`.

---

### 7.7 `rspm-cli` — Binary Entry

**Tujuan**: CLI parsing dengan clap, mapping ke client API.

```
crates/rspm-cli/
└── src/
    ├── main.rs
    ├── commands/
    │   ├── mod.rs
    │   ├── start.rs
    │   ├── stop.rs
    │   ├── restart.rs
    │   ├── reload.rs
    │   ├── delete.rs
    │   ├── list.rs
    │   ├── logs.rs
    │   ├── monit.rs
    │   ├── describe.rs
    │   ├── jlist.rs
    │   ├── save.rs
    │   ├── resurrect.rs
    │   ├── startup.rs
    │   ├── dump.rs
    │   ├── flush.rs
    │   ├── ping.rs
    │   ├── kill.rs
    │   ├── update.rs
    │   ├── scale.rs
    │   └── module.rs
    ├── format/
    │   ├── table.rs       # comfy-table for `list`
    │   ├── color.rs       # owo-colors theme
    │   └── json.rs
    └── cli.rs             # clap derive struct
```

**Fungsi utama**:

- `cli::Cli` (clap derive) dengan subcommand untuk tiap operasi.
- `commands::start::run(args, client) -> Result<()>`.
- `format::table::render_process_list(list: &[ProcessInfo]) -> String`.

**PM2 Refs**: `pm2/lib/binaries/CLI.js` (semua subcommand).

---

### 7.8 `rspm-watcher` — File Watcher

**Tujuan**: Watch source file, trigger restart on change.

```
crates/rspm-watcher/
└── src/
    ├── lib.rs
    ├── watcher.rs         # AppWatcher (per app)
    ├── matcher.rs         # globset for include/ignore
    └── debounce.rs        # collapse rapid events
```

**Fungsi utama**:

- `AppWatcher::new(cwd: &Path, patterns: WatchSpec, ignore: &[String]) -> Result<Self>`.
- `AppWatcher::events(&mut self) -> impl Stream<Item=WatchEvent>`.

**PM2 Refs**: `pm2/lib/Watcher.js` (chokidar wrapper).

---

### 7.9 `rspm-monitor` — Resource Sampler

**Tujuan**: Sample CPU% dan MEM bytes setiap child PID.

```
crates/rspm-monitor/
└── src/
    ├── lib.rs
    ├── sampler.rs         # poll /proc/PID/stat
    └── aggregator.rs      # rolling average
```

**Fungsi utama**:

- `Sampler::new(interval: Duration) -> Self`.
- `Sampler::sample(&mut self, pids: &[u32]) -> Vec<(u32, CpuSample, MemSample)>`.

**PM2 Refs**: `pm2/lib/God/SystemData.js`, `pm2/lib/God/ActionMethods.js:getMonitorData`.

---

### 7.10 `rspm-logs` — Log Writer & Rotation

**Tujuan**: Tulis stdout/stderr child ke file. Support rotation.

```
crates/rspm-logs/
└── src/
    ├── lib.rs
    ├── writer.rs          # LogWriter (per app)
    ├── timestamp.rs       # strftime prefix
    ├── rotator.rs         # size/time-based rotation
    └── tail.rs            # follow file (for `logs` command)
```

**Fungsi utama**:

- `LogWriter::new(path: &Path, opts: LogOpts) -> Result<Self>`.
- `LogWriter::write_line(&mut self, line: &[u8]) -> Result<()>`.
- `Rotator::rotate_if_needed(&mut self) -> Result<()>`.
- `tail::follow(path: &Path, from_end: bool) -> impl Stream<Item=String>`.

**PM2 Refs**: `pm2/lib/Utility.js` (timestamp helper), `pm2/lib/API/Log.js` (tail UI).

---

### 7.11 `rspm-cluster` — Universal Cluster (SO_REUSEPORT)

**Tujuan**: Pre-bind listening socket dengan `SO_REUSEPORT`, share fd ke N child via `SCM_RIGHTS`.

```
crates/rspm-cluster/
└── src/
    ├── lib.rs
    ├── reuseport.rs       # bind socket with SO_REUSEPORT
    └── fd_passing.rs      # send fd to child via unix socket SCM_RIGHTS
```

**Fungsi utama**:

- `bind_reuseport(addr: SocketAddr) -> Result<RawFd>`.
- `send_fd_to_child(socket: &UnixStream, fd: RawFd) -> Result<()>`.

**Strategi v0.0.1**: Set env `RSPM_CLUSTER_FD` di child, child membaca fd dari env (passing via inherited fd). Child app (Node/Python/Go) harus dukung `SO_REUSEPORT` natively (mayoritas runtime modern sudah). Untuk Node.js, set `NODE_APP_INSTANCE` agar app load index.js yang `listen(port)` — kernel handle load balancing via REUSEPORT.

**PM2 Refs**: PM2 pakai Node.js `cluster` module yang tidak universal. Plan kita ganti dengan REUSEPORT (D4).

---

### 7.12 `rspm-modules` — Modules

**Tujuan**: Install/uninstall module (npm package atau git URL) sebagai long-running process.

```
crates/rspm-modules/
└── src/
    ├── lib.rs
    ├── installer.rs       # npm install / git clone
    ├── registry.rs        # state modul terinstall
    └── lifecycle.rs       # start/stop modul
```

**PM2 Refs**: `pm2/lib/API/Modules/`.

---

### 7.13 `rspm-startup` — System Init Script Generator

**Tujuan**: Generate unit file untuk systemd/openrc/sysv.

```
crates/rspm-startup/
└── src/
    ├── lib.rs
    ├── detect.rs          # detect init system (systemd/openrc/upstart/sysv)
    ├── generator.rs       # tera render
    └── templates/
        ├── systemd.tera
        ├── openrc.tera
        └── sysv.tera
```

**Fungsi utama**:

- `detect_init_system() -> InitSystem`.
- `generate(init: InitSystem, ctx: &StartupCtx) -> String`.
- `install(unit_text: &str, path: &Path) -> Result<()>` (caller harus sudo).

**PM2 Refs**: `pm2/lib/API/Startup.js`, `pm2/lib/templates/init-scripts/*.tpl`.

---

### 7.14 `rspm-deploy` — SSH Deploy

**Tujuan**: Deploy app ke server remote via SSH (subset PM2 deploy).

```
crates/rspm-deploy/
└── src/
    ├── lib.rs
    ├── ssh.rs             # russh client wrapper
    ├── git.rs             # git checkout/pull on remote
    └── hooks.rs           # pre/post-deploy hooks
```

**PM2 Refs**: `pm2/lib/API/Deploy.js`, dep `pm2-deploy`.

---

### 7.15 `rspm-dashboard` — TUI (pm2 monit)

**Tujuan**: Interactive terminal UI.

```
crates/rspm-dashboard/
└── src/
    ├── lib.rs
    ├── app.rs             # state
    ├── ui.rs              # ratatui render
    └── input.rs           # crossterm keymap
```

**PM2 Refs**: `pm2/lib/API/Monit.js`.

---

### 7.16 `rspm-http` — Optional HTTP API

**Tujuan**: Expose subset operasi via HTTP (axum).

```
crates/rspm-http/
└── src/
    ├── lib.rs
    ├── server.rs
    └── routes/
        ├── processes.rs
        └── logs.rs
```

**Activation**: `rspm http --port 9615` (mirip `pm2 web`).

---

## 8. Task Breakdown (20 Phases, ~120 Task Atomik)

> Format: **T{phase}.{idx}** — judul. Berisi: Goal, File, PM2 Ref, Flow Pseudo-code, DoD (Definition of Done).

### Phase 0 — Bootstrap Workspace

#### T0.1 Inisialisasi cargo workspace

- **Goal**: Set up root workspace + 16 crate kosong.
- **File**: `Cargo.toml` (root), `crates/*/Cargo.toml`, `crates/*/src/lib.rs`.
- **PM2 Ref**: tidak ada.
- **Flow**:
  ```
  cargo new --lib crates/rspm-core
  ... ulangi untuk semua crate ...
  Tulis [workspace] members = [...] di root Cargo.toml.
  Tambah [workspace.package] version="0.0.1", edition="2024", license="MIT".
  ```
- **DoD**: `cargo build --workspace` sukses (lib kosong).

#### T0.2 Setup rust-toolchain + deny + clippy config

- **File**: `rust-toolchain.toml`, `deny.toml`, `clippy.toml`, `.cargo/config.toml`.
- **Flow**:
  ```
  rust-toolchain.toml: channel = "1.75"
  deny.toml: licenses allow MIT/Apache-2.0/BSD-*, advisories deny RUSTSEC
  clippy.toml: msrv = "1.75"
  ```
- **DoD**: `cargo clippy -- -D warnings` 0 issue.

#### T0.3 CLAUDE.md project root

- **File**: `/home/zdanysfa/rspm/CLAUDE.md`.
- **Flow**: Copy konten Section 5 plan ini.
- **DoD**: File ada di root.

#### T0.4 README placeholder + LICENSE MIT + .gitignore Rust

- **File**: `README.md`, `LICENSE`, `.gitignore`.
- **DoD**: 3 file ada.

#### T0.5 CI skeleton (GitHub Actions)

- **File**: `.github/workflows/ci.yml`.
- **Flow**: matrix [ubuntu-latest], steps: fmt, clippy, test, deny.
- **DoD**: workflow file valid.

---

### Phase 1 — rspm-core Foundation

#### T1.1 Definisi RspmError

- **Goal**: Hierarchical error pakai thiserror.
- **File**: `crates/rspm-core/src/error.rs`.
- **Flow**:
  ```rust
  #[derive(thiserror::Error, Debug)]
  pub enum RspmError {
      #[error("io: {0}")] Io(#[from] std::io::Error),
      #[error("config: {0}")] Config(String),
      #[error("protocol: {0}")] Protocol(String),
      #[error("ipc: {0}")] Ipc(String),
      #[error("process not found: {0}")] ProcessNotFound(String),
      ...
  }
  pub type Result<T> = std::result::Result<T, RspmError>;
  ```
- **DoD**: test enum bisa dibuat & display.

#### T1.2 Constants

- **File**: `crates/rspm-core/src/constants.rs`.
- **Flow**:
  ```rust
  pub const KILL_TIMEOUT_MS: u64 = 1600;
  pub const WORKER_INTERVAL_MS: u64 = 30_000;
  pub const MIN_UPTIME_MS: u64 = 1_000;
  pub const MAX_RESTARTS_DEFAULT: u32 = 16;
  pub const LISTEN_TIMEOUT_MS: u64 = 8_000;
  pub const DEFAULT_INSTANCE_VAR: &str = "NODE_APP_INSTANCE";
  ...
  ```
- **PM2 Ref**: `pm2/constants.js` (port semua nilai).
- **DoD**: nilai identik dengan `pm2/constants.js`.

#### T1.3 RspmHome path resolver

- **File**: `crates/rspm-core/src/paths.rs`.
- **Flow**:
  ```rust
  pub struct RspmHome { root: PathBuf }
  impl RspmHome {
      pub fn from_env() -> Result<Self> {
          let root = std::env::var_os("RSPM_HOME")
              .map(PathBuf::from)
              .unwrap_or_else(|| dirs::home_dir().unwrap().join(".rspm"));
          std::fs::create_dir_all(&root)?;
          // create subdirs: pids, logs, modules
          Ok(Self { root })
      }
      pub fn rpc_socket(&self) -> PathBuf { self.root.join("rpc.sock") }
      pub fn pub_socket(&self) -> PathBuf { self.root.join("pub.sock") }
      pub fn dump_file(&self) -> PathBuf { self.root.join("dump.rspm") }
      pub fn log_dir(&self) -> PathBuf { self.root.join("logs") }
      pub fn pid_dir(&self) -> PathBuf { self.root.join("pids") }
      pub fn daemon_pid_file(&self) -> PathBuf { self.root.join("pm2.pid") }
      pub fn daemon_log_file(&self) -> PathBuf { self.root.join("pm2.log") }
  }
  ```
- **PM2 Ref**: `pm2/paths.js`.
- **DoD**: test override via env var, default path benar.

#### T1.4 Types: AppConfig

- **File**: `crates/rspm-core/src/types/app.rs`.
- **Flow**: struct dengan semua field di tabel section 7.3 + `#[serde(default = "defaults::...")]`.
- **DoD**: round-trip JSON serialize/deserialize sukses.

#### T1.5 Types: ProcessInfo + ProcessStatus

- **File**: `crates/rspm-core/src/types/process.rs`.
- **Flow**:
  ```rust
  pub enum ProcessStatus { Online, Stopping, Stopped, Errored, Launching, OneLaunch, Waiting }
  pub struct ProcessInfo {
      pub pm_id: u32,
      pub name: String,
      pub pid: Option<u32>,
      pub status: ProcessStatus,
      pub restart_time: u32,
      pub unstable_restarts: u32,
      pub created_at: DateTime<Utc>,
      pub uptime: Option<Duration>,
      pub cpu: f32,
      pub memory: u64,
      pub instance_id: Option<u32>,
      pub app: AppConfig,
  }
  ```
- **PM2 Ref**: `pm2/lib/God.js` (state struct).
- **DoD**: serialisasi JSON OK.

#### T1.6 Types: ExecutionMode + InstanceCount + WatchSpec

- **File**: `crates/rspm-core/src/types/app.rs` (lanjutan).
- **Flow**:
  ```rust
  pub enum ExecutionMode { Fork, Cluster }
  pub enum InstanceCount { N(u32), Max, MaxMinusOne(u32) } // PM2: "max" / -1
  pub enum WatchSpec { Disabled, Enabled, Patterns(Vec<String>) }
  ```
- **DoD**: deserialize "max", -1, true/false/array semua benar.

#### T1.7 Defaults module

- **File**: `crates/rspm-core/src/defaults.rs`.
- **Flow**: fn-fn untuk `#[serde(default = "...")]`.
- **DoD**: kompilasi.

#### T1.8 Version info (build-time)

- **File**: `crates/rspm-core/src/version.rs` + `build.rs`.
- **Flow**: ambil `CARGO_PKG_VERSION`, embed git commit hash via `vergen`.
- **DoD**: `rspm --version` (nanti) tampil "0.0.1+abc1234".

---

### Phase 2 — rspm-protocol

#### T2.1 Frame codec

- **File**: `crates/rspm-protocol/src/frame.rs`.
- **Flow**:
  ```rust
  pub struct FrameCodec;
  impl Decoder for FrameCodec {
      type Item = Bytes;
      fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Bytes>>;
  }
  impl Encoder<Bytes> for FrameCodec { ... }
  // Format: [u32 BE length][payload bytes]
  ```
- **DoD**: round-trip test.

#### T2.2 Request enum

- **File**: `crates/rspm-protocol/src/request.rs`.
- **Flow**:
  ```rust
  #[derive(Serialize, Deserialize)]
  #[serde(tag = "method", content = "params")]
  pub enum Request {
      Ping,
      GetVersion,
      List,
      StartApp(AppConfig),
      StartApps(Vec<AppConfig>),
      StopByName(String),
      StopById(u32),
      RestartByName(String),
      RestartById(u32),
      ReloadByName(String),
      DeleteByName(String),
      DeleteAll,
      Scale { name: String, instances: u32 },
      SendSignal { id: u32, signal: String },
      GetMonitorData,
      GetSystemData,
      Describe(u32),
      DumpProcessList,
      SaveProcessList,
      ReloadLogs,
      Flush(Option<String>),
      NotifyKill,
      ...
  }
  ```
- **PM2 Ref**: `pm2/lib/God/ActionMethods.js` (semua method name).
- **DoD**: semua variant test deserialisasi.

#### T2.3 Response enum

- **File**: `crates/rspm-protocol/src/response.rs`.
- **Flow**: mirror Request, tiap variant return value.
  ```rust
  pub enum Response {
      Ack,
      Version(String),
      ProcessList(Vec<ProcessInfo>),
      Started(Vec<ProcessInfo>),
      Stopped(Vec<u32>),
      MonitorData(MonitorSnapshot),
      Error { code: String, message: String },
      ...
  }
  ```
- **DoD**: serialize/deserialize OK.

#### T2.4 Event enum + PROTOCOL_VERSION const

- **File**: `crates/rspm-protocol/src/event.rs`, `version.rs`.
- **Flow**:
  ```rust
  pub enum Event {
      ProcessOnline { pm_id: u32, pid: u32, name: String },
      ProcessExit { pm_id: u32, code: i32, signal: Option<String> },
      LogLine { pm_id: u32, stream: LogStream, line: String, at: DateTime<Utc> },
      Reload { pm_id: u32 },
      ...
  }
  pub const PROTOCOL_VERSION: u32 = 1;
  ```
- **DoD**: semua variant tested.

#### T2.5 Handshake message

- **File**: `crates/rspm-protocol/src/handshake.rs`.
- **Flow**:
  ```rust
  pub struct Handshake { pub protocol_version: u32, pub client_version: String }
  pub struct HandshakeAck { pub protocol_version: u32, pub daemon_version: String }
  ```
- **DoD**: round-trip.

---

### Phase 3 — rspm-ipc

#### T3.1 UDS bind helper + permission

- **File**: `crates/rspm-ipc/src/server.rs`.
- **Flow**:
  ```
  Hapus file socket lama kalau ada (cek stale).
  bind UnixListener.
  chmod 0600 file socket.
  ```
- **DoD**: bisa bind 2x berturut-turut.

#### T3.2 IpcServer accept loop

- **File**: `crates/rspm-ipc/src/server.rs`.
- **Flow**:
  ```rust
  pub trait Handler: Send + Sync + 'static {
      fn handle(&self, req: Request) -> BoxFuture<Result<Response>>;
  }
  pub struct IpcServer { rpc: UnixListener, bus: Bus }
  impl IpcServer {
      pub async fn run<H: Handler>(self, h: Arc<H>) {
          loop {
              let (stream, _) = self.rpc.accept().await?;
              tokio::spawn(handle_conn(stream, h.clone()));
          }
      }
  }
  async fn handle_conn(stream, h) {
      let mut framed = Framed::new(stream, FrameCodec);
      while let Some(frame) = framed.next().await {
          let req: Request = serde_json::from_slice(&frame?)?;
          let resp = h.handle(req).await;
          framed.send(serde_json::to_vec(&resp)?.into()).await?;
      }
  }
  ```
- **DoD**: echo test passing.

#### T3.3 PubSubBus

- **File**: `crates/rspm-ipc/src/bus.rs`.
- **Flow**:
  ```rust
  pub struct Bus { tx: broadcast::Sender<Event> }
  impl Bus {
      pub fn new(cap: usize) -> Self { let (tx, _) = broadcast::channel(cap); Self { tx } }
      pub fn publish(&self, e: Event) { let _ = self.tx.send(e); }
      pub fn subscribe(&self) -> broadcast::Receiver<Event> { self.tx.subscribe() }
  }
  ```
- **DoD**: 2 subscriber dapat event yang sama.

#### T3.4 Pub socket server (broadcast to all connected)

- **File**: `crates/rspm-ipc/src/server.rs` (lanjut).
- **Flow**: bind `pub.sock`, accept, untuk tiap conn `bus.subscribe()` lalu forward ke socket.
- **DoD**: client dapat event yang dipublish daemon.

#### T3.5 IpcClient

- **File**: `crates/rspm-ipc/src/client.rs`.
- **Flow**:
  ```rust
  pub struct IpcClient { framed: Framed<UnixStream, FrameCodec> }
  impl IpcClient {
      pub async fn connect(path: &Path) -> Result<Self>;
      pub async fn call(&mut self, req: Request) -> Result<Response>;
  }
  ```
- **DoD**: integration test client↔server.

#### T3.6 Event subscriber client

- **File**: `crates/rspm-ipc/src/client.rs` (lanjut).
- **Flow**: `EventSubscriber::connect(pub_path)` returns Stream<Event>.
- **DoD**: subscribe + receive 5 event berturut-turut.

#### T3.7 Handshake protocol enforce

- **File**: `crates/rspm-ipc/src/handshake.rs`.
- **Flow**: pertama kali connect, kirim Handshake. Daemon balas Ack. Mismatch → tutup koneksi.
- **DoD**: test mismatch ditolak.

---

### Phase 4 — rspm-config

#### T4.1 AppConfigInput (raw, sebelum normalize)

- **File**: `crates/rspm-config/src/schema.rs`.
- **Flow**:
  ```rust
  #[derive(Deserialize)]
  pub struct AppConfigInput {
      pub name: Option<String>,
      pub script: PathBuf,
      pub args: Option<Args>,           // String / Vec<String>
      pub instances: Option<Instances>, // i32 / "max"
      pub exec_mode: Option<String>,    // "fork" | "cluster"
      ...
  }
  ```
- **DoD**: parse minimal JSON valid.

#### T4.2 Loader auto-detect format

- **File**: `crates/rspm-config/src/loader.rs`.
- **Flow**:
  ```
  match path extension {
      "toml" => parse_toml,
      "yml"|"yaml" => parse_yaml,
      "json" => parse_json,
      "js"  => parse_ecosystem (boa),
      _ => err
  }
  ```
- **DoD**: 4 fixture parse OK.

#### T4.3 Ecosystem.config.js parser via boa

- **File**: `crates/rspm-config/src/format/ecosystem.rs`.
- **Flow**:
  ```
  let mut ctx = boa_engine::Context::default();
  // Inject a stub module object so the user file can assign module.exports = {...}
  ctx.parse_source(...).execute("var module = {exports:{}};");
  ctx.parse_source(...).execute(js_src);
  let exports = ctx.global_object().get("module").get("exports");
  let apps = exports.get("apps");  // JsArray
  for each app -> serde_json::Value -> AppConfigInput
  ```
  > Note: boa API name is `Context::eval_script` / `parse_source` etc. Pseudo-code above abstracts the call site; the actual API depends on boa version pinned in `Cargo.toml`.
- **PM2 Ref**: `pm2/lib/Common.js:resolveAppPaths`.
- **DoD**: parse `examples/ecosystem.config.js` real PM2 sample sukses.

#### T4.4 Normalize defaults

- **File**: `crates/rspm-config/src/normalize.rs`.
- **Flow**:
  ```
  fn apply_defaults(i: AppConfigInput, cwd: &Path) -> AppConfig {
      name = i.name.unwrap_or_else(|| derive_from_script(&i.script));
      cwd = i.cwd.unwrap_or(cwd.to_owned());
      execution_mode = parse_exec_mode(i.exec_mode).unwrap_or(ExecutionMode::Fork);
      kill_timeout_ms = i.kill_timeout.unwrap_or(KILL_TIMEOUT_MS);
      ... // semua field
  }
  ```
- **PM2 Ref**: `pm2/lib/Common.js:prepareAppConf`.
- **DoD**: golden file test (compare struct hasil dengan expected).

#### T4.5 Env var expansion `${VAR}`

- **File**: `crates/rspm-config/src/env_expand.rs`.
- **Flow**: regex `\$\{([A-Z_][A-Z0-9_]*)\}` → ganti dari `EnvMap`.
- **DoD**: nested $ tidak infinite loop.

#### T4.6 Validasi cross-field

- **File**: `crates/rspm-config/src/normalize.rs`.
- **Flow**: cluster mode butuh `instances`, script harus exist, dll.
- **DoD**: error message jelas.

#### T4.7 Multi-env override (env_production, env_development)

- **File**: `crates/rspm-config/src/normalize.rs`.
- **Flow**: jika user pass `--env production`, merge `env_production` ke `env`.
- **PM2 Ref**: `pm2/lib/Common.js:resolveEnvironment`.
- **DoD**: env override applied correctly.

---

### Phase 5 — rspm-daemon Core

#### T5.1 Daemon binary entry

- **File**: `crates/rspm-daemon/src/main.rs`.
- **Flow**:
  ```
  fn main() {
      let home = RspmHome::from_env()?;
      acquire_pid_lock(&home.daemon_pid_file())?;
      setup_tracing(&home.daemon_log_file())?;
      let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
      rt.block_on(async { God::run(home).await })
  }
  ```
- **PM2 Ref**: `pm2/lib/Daemon.js:processStateHandler` + entry block.
- **DoD**: binary boot, tulis pid file, listen socket.

#### T5.2 PID file lock

- **File**: `crates/rspm-daemon/src/pid_file.rs`.
- **Flow**: open with O_CREAT|O_EXCL, write pid. Jika ada → cek apakah pid masih hidup (kill 0). Stale → hapus.
- **DoD**: 2 daemon tidak bisa start bersamaan.

#### T5.3 God singleton struct

- **File**: `crates/rspm-daemon/src/god.rs`.
- **Flow**:
  ```rust
  pub struct God {
      pub home: RspmHome,
      pub next_pm_id: u32,
      pub processes: BTreeMap<u32, ManagedProcess>,
      pub bus: Bus,
      pub config: GodConfig,
  }
  pub struct ManagedProcess {
      pub info: ProcessInfo,
      pub child: Option<tokio::process::Child>,
      pub log_writer: LogWriter,
      pub watcher: Option<AppWatcher>,
      pub restart_history: VecDeque<Instant>,
  }
  ```
- **DoD**: kompilasi.

#### T5.4 Spawn fork mode child

- **File**: `crates/rspm-daemon/src/supervisor/fork_mode.rs`.
- **Flow**:
  ```
  Build tokio::process::Command:
      program = app.interpreter.unwrap_or(detect_interpreter(&app.script))
      args = [interpreter_args..., script.to_string(), args...]
      cwd(app.cwd)
      env_clear if app.detached_env, else inherit + apply app.env
      stdout(Stdio::piped()), stderr(Stdio::piped()), stdin(Stdio::null())
  Spawn. Get child.id(). Capture stdout/stderr -> log_writer.
  Update ProcessInfo: pid, status=Online, created_at=now.
  bus.publish(Event::ProcessOnline {...})
  Spawn task: child.wait().await -> God::on_exit(pm_id, status)
  ```
- **PM2 Ref**: `pm2/lib/God/ForkMode.js`.
- **DoD**: jalankan `node -e "setInterval(()=>{},1000)"`, list shows online.

#### T5.5 Capture stdout/stderr → log files

- **File**: `crates/rspm-daemon/src/log_capture.rs`.
- **Flow**:
  ```
  let mut reader = BufReader::new(child.stdout.take().unwrap()).lines();
  while let Some(line) = reader.next_line().await? {
      log_writer.write_line(LogStream::Out, &line)?;
      bus.publish(Event::LogLine {...});
  }
  ```
- **DoD**: log file ter-update real-time.

#### T5.6 on_exit handler dengan restart policy

- **File**: `crates/rspm-daemon/src/supervisor/lifecycle.rs`.
- **Flow**:
  ```
  fn on_exit(god, pm_id, status):
      proc = god.processes.get_mut(pm_id).unwrap();
      uptime = now - proc.info.created_at;
      if uptime < min_uptime_ms:
          proc.info.unstable_restarts += 1;
      if proc.info.unstable_restarts >= max_restarts:
          proc.info.status = Errored;
          bus.publish(ProcessErrored);
          return;
      if !app.auto_restart:
          proc.info.status = Stopped;
          return;
      delay = exp_backoff(proc.info.restart_time, app.restart_delay);
      tokio::spawn(after delay -> god.start_again(pm_id));
      proc.info.restart_time += 1;
  ```
- **PM2 Ref**: `pm2/lib/God.js:handleExit`.
- **DoD**: child yang exit langsung di-restart sampai max_restarts.

#### T5.7 Handler dispatch dari IPC ke God methods

- **File**: `crates/rspm-daemon/src/handler.rs`.
- **Flow**:
  ```rust
  impl Handler for GodHandler {
      async fn handle(&self, req: Request) -> Result<Response> {
          let mut g = self.god.lock().await;
          match req {
              Request::Ping => Ok(Response::Ack),
              Request::StartApp(cfg) => g.start_app(cfg).await.map(Response::Started),
              ...
          }
      }
  }
  ```
- **DoD**: e2e: client kirim Ping → dapat Ack.

#### T5.8 Graceful shutdown signal handling

- **File**: `crates/rspm-daemon/src/god.rs`.
- **Flow**:
  ```
  tokio::select! {
      _ = signal::ctrl_c() => shutdown,
      _ = signal_unix::signal(SIGTERM) => shutdown,
  }
  for each process: send_stop(SIGINT) lalu wait kill_timeout, kalau masih hidup SIGKILL.
  flush log_writer.
  cleanup socket files.
  exit 0.
  ```
- **PM2 Ref**: `pm2/lib/Daemon.js:gracefullExit`.
- **DoD**: SIGTERM → semua child mati bersih.

---

### Phase 6 — Handlers (lanjut)

#### T6.1 Handler: stop (by id / name)

- **File**: `crates/rspm-daemon/src/handlers/stop.rs`.
- **Flow**:
  ```
  set proc.info.status = Stopping (so on_exit won't restart)
  send SIGINT to proc.pid via nix::sys::signal
  spawn timer kill_timeout_ms: if still alive -> SIGKILL
  ```
- **PM2 Ref**: `pm2/lib/God/ActionMethods.js:stopProcessId`.
- **DoD**: stop test: process exits dalam ≤ 2s.

#### T6.2 Handler: restart

- **File**: `crates/rspm-daemon/src/handlers/restart.rs`.
- **Flow**: stop → wait exit → respawn → return info.
- **DoD**: restart_time bertambah 1.

#### T6.3 Handler: delete

- **File**: `crates/rspm-daemon/src/handlers/delete.rs`.
- **Flow**: stop dulu, lalu `god.processes.remove(pm_id)`.
- **DoD**: setelah delete, list tidak menampilkan.

#### T6.4 Handler: list

- **File**: `crates/rspm-daemon/src/handlers/list.rs`.
- **Flow**: clone `processes.values().map(|p| p.info.clone()).collect()`.
- **DoD**: returned list matches state.

#### T6.5 Handler: describe (detail single process)

- **File**: `crates/rspm-daemon/src/handlers/describe.rs`.
- **Flow**: cari by id/name, return ProcessInfo lengkap + env + path log.
- **PM2 Ref**: `pm2/lib/God/ActionMethods.js:getProcessById`.
- **DoD**: berisi semua field.

#### T6.6 Handler: monit data

- **File**: `crates/rspm-daemon/src/handlers/monit.rs`.
- **Flow**: untuk setiap process online, ambil sampel terbaru dari `rspm-monitor`.
- **DoD**: cpu% dan mem byte populated.

#### T6.7 Handler: send signal arbitrary

- **File**: `crates/rspm-daemon/src/handlers/signal.rs`.
- **Flow**: parse signal name → `nix::sys::signal::Signal`, kirim ke pid.
- **DoD**: SIGUSR1 dapat dikirim.

#### T6.8 Handler: scale (resize cluster)

- **File**: `crates/rspm-daemon/src/handlers/restart.rs` (atau scale.rs).
- **Flow**: if target > current → spawn (target-current) instance. If < → stop instance ID terakhir.
- **PM2 Ref**: `pm2/lib/God/ActionMethods.js:scale`.
- **DoD**: scale 2→4→1 berfungsi.

#### T6.9 Handler: dump (save process list ke disk)

- **File**: `crates/rspm-daemon/src/handlers/dump.rs`.
- **Flow**: serialize `Vec<AppConfig>` (bukan ProcessInfo) ke `dump_file()` sebagai JSON pretty.
- **PM2 Ref**: `pm2/lib/God/ActionMethods.js:dumpProcessList`.
- **DoD**: file ada + valid JSON.

#### T6.10 Handler: resurrect (load dump → start)

- **File**: `crates/rspm-daemon/src/handlers/dump.rs` (lanjut).
- **Flow**: baca dump_file, loop apps, panggil start_app.
- **DoD**: setelah kill daemon + resurrect, semua app hidup lagi.

#### T6.11 Handler: flush logs

- **File**: `crates/rspm-daemon/src/handlers/dump.rs` (atau logs.rs).
- **Flow**: untuk tiap proc, truncate log file ke 0.
- **DoD**: log files kosong setelah flush.

#### T6.12 Handler: reloadLogs (rotate)

- **Flow**: tutup file handle saat ini, rename ke .1, buka file baru. Useful untuk logrotate.
- **DoD**: file path lama jadi .1, baru kosong.

---

### Phase 7 — rspm-client + rspm-cli MVP

#### T7.1 Client launcher (auto-spawn daemon)

- **File**: `crates/rspm-client/src/daemon_launcher.rs`.
- **Flow**:
  ```
  if !rpc_socket.exists() OR can't connect:
      spawn rspm_daemon binary (detached, setsid)
      wait until socket exists (poll 100ms, timeout 5s)
  ```
- **PM2 Ref**: `pm2/lib/Client.js:launchDaemon`.
- **DoD**: dari fresh state, client connect berhasil.

#### T7.2 RspmClient high-level API

- **File**: `crates/rspm-client/src/api.rs`.
- **Flow**: tipis di atas `IpcClient`.
- **DoD**: integration test panggil semua method MVP.

#### T7.3 CLI: clap derive root

- **File**: `crates/rspm-cli/src/cli.rs`.
- **Flow**:
  ```rust
  #[derive(Parser)]
  pub struct Cli {
      #[command(subcommand)]
      pub command: Command,
      #[arg(long, global=true)]
      pub home: Option<PathBuf>,
      #[arg(long, global=true)]
      pub silent: bool,
  }
  #[derive(Subcommand)]
  pub enum Command {
      Start(StartArgs),
      Stop(StopArgs),
      ...
  }
  ```
- **DoD**: `rspm --help` jalan.

#### T7.4 CLI: start command

- **File**: `crates/rspm-cli/src/commands/start.rs`.
- **Flow**:
  ```
  parse args (path config OR script + flags inline)
  build AppConfig(s)
  client.start_apps(apps).await
  render table hasil
  ```
- **DoD**: `rspm start app.js --name foo` jalan.

#### T7.5 CLI: stop / restart / delete / list

- **File**: `crates/rspm-cli/src/commands/{stop,restart,delete,list}.rs`.
- **DoD**: 4 command e2e OK.

#### T7.6 CLI: logs (stream + tail)

- **File**: `crates/rspm-cli/src/commands/logs.rs`.
- **Flow**:
  ```
  if --lines N: tail last N from file
  subscribe bus EventLogLine, filter by name/id, print with color (out=normal, err=red)
  ```
- **DoD**: live stream terlihat.

#### T7.7 CLI: jlist (JSON output)

- **File**: `crates/rspm-cli/src/commands/jlist.rs`.
- **DoD**: keluaran valid JSON pipe-able.

#### T7.8 CLI: ping

- **File**: `crates/rspm-cli/src/commands/ping.rs`.
- **DoD**: "pong" / latency ms ditampilkan.

#### T7.9 CLI: kill (stop daemon)

- **File**: `crates/rspm-cli/src/commands/kill.rs`.
- **Flow**: send `NotifyKill` → daemon shutdown → tunggu pid file hilang.
- **DoD**: socket file hilang setelah kill.

---

### Phase 8 — Worker (Background)

#### T8.1 Worker tick loop

- **File**: `crates/rspm-daemon/src/worker.rs`.
- **Flow**:
  ```
  every WORKER_INTERVAL_MS:
      sample_metrics_for_all()
      check_max_memory_restart()
      cron_restart_check()
      reload_if_due()
  ```
- **PM2 Ref**: `pm2/lib/Worker.js`.
- **DoD**: tick terbukti jalan.

#### T8.2 Max memory restart

- **File**: `crates/rspm-daemon/src/worker.rs` (lanjut).
- **Flow**: jika `info.memory > app.max_memory_restart`, panggil restart handler.
- **DoD**: integration test dengan child yang inflate memory.

#### T8.3 Cron restart

- **File**: `crates/rspm-daemon/src/worker.rs`.
- **Flow**: parse cron spec dengan crate `cron`, hitung next, jika lewat → restart.
- **PM2 Ref**: `pm2/lib/Worker.js:_runCron`.
- **DoD**: spec `*/1 * * * *` restart tiap menit.

---

### Phase 9 — rspm-monitor

#### T9.1 Sampler from /proc/[pid]/stat

- **File**: `crates/rspm-monitor/src/sampler.rs`.
- **Flow**: pakai `sysinfo` crate `System::refresh_process` per pid. Hitung delta cpu time → %.
- **DoD**: test memory match dengan `ps -o rss`.

#### T9.2 Aggregator rolling window

- **File**: `crates/rspm-monitor/src/aggregator.rs`.
- **Flow**: ring buffer 10 sample, rata-rata cpu.
- **DoD**: avg(samples) konsisten.

---

### Phase 10 — rspm-watcher

#### T10.1 AppWatcher dengan notify crate

- **File**: `crates/rspm-watcher/src/watcher.rs`.
- **Flow**:
  ```
  notify::recommended_watcher
  watch app.cwd recursive
  filter dengan globset (watch patterns) + exclude (ignore_watch + node_modules + .git default)
  debounce 200ms
  emit WatchEvent
  ```
- **PM2 Ref**: `pm2/lib/Watcher.js`.
- **DoD**: ubah file → 1 event muncul.

#### T10.2 Integrate watcher → trigger restart

- **File**: `crates/rspm-daemon/src/god.rs`.
- **Flow**: saat start_app dengan watch enabled, spawn watcher task. On event → call restart handler.
- **DoD**: ubah file → child restart.

---

### Phase 11 — rspm-logs Rotation & Format

#### T11.1 LogWriter dengan optional timestamp prefix

- **File**: `crates/rspm-logs/src/writer.rs`.
- **Flow**:
  ```
  if app.prefix_timestamp: prepend now().format(app.log_date_format)
  if app.merge_logs: hilangkan instance suffix
  write to file with O_APPEND
  ```
- **PM2 Ref**: `pm2/lib/Utility.js`.
- **DoD**: format match PM2.

#### T11.2 Rotator size-based

- **File**: `crates/rspm-logs/src/rotator.rs`.
- **Flow**:
  ```
  on each write check size; if > limit: rename .log -> .1.log -> .2.log ... up to max
  ```
- **DoD**: rotate happen.

#### T11.3 Tail follow stream

- **File**: `crates/rspm-logs/src/tail.rs`.
- **Flow**: open file, seek end (or -N lines), inotify on file path (rotation aware via inode change).
- **DoD**: stream tetap jalan setelah rotate.

---

### Phase 12 — Cluster Mode (SO_REUSEPORT)

#### T12.1 bind_reuseport helper

- **File**: `crates/rspm-cluster/src/reuseport.rs`.
- **Flow**:
  ```
  let fd = socket(AF_INET, SOCK_STREAM, 0);
  setsockopt(fd, SO_REUSEADDR, 1);
  setsockopt(fd, SO_REUSEPORT, 1);
  bind(fd, addr);
  listen(fd, backlog);
  return fd
  ```
- **DoD**: 2 process bind sama port sukses.

#### T12.2 Cluster supervisor: spawn N child sharing socket

- **File**: `crates/rspm-daemon/src/supervisor/cluster_mode.rs`.
- **Flow**:
  ```
  for i in 0..instances:
      env = base_env + { NODE_APP_INSTANCE: i, RSPM_INSTANCE_ID: i }
      spawn child with inherited stdin null
      (no fd passing kalau app pakai REUSEPORT natively)
  ```
- **PM2 Ref**: `pm2/lib/God/ClusterMode.js` (kita pakai pattern berbeda, lebih universal).
- **DoD**: 4 instance node listen 3000 paralel.

#### T12.3 Soft reload (zero downtime)

- **File**: `crates/rspm-daemon/src/handlers/reload.rs`.
- **Flow**:
  ```
  for each instance in rolling order:
      spawn replacement instance (instance_id sama)
      wait until 'ready' (wait_ready: process msg 'ready' OR after listen_timeout)
      send SIGINT to old instance
      wait exit (kill_timeout)
  ```
- **PM2 Ref**: `pm2/lib/God/Reload.js`.
- **DoD**: HTTP load test selama reload — 0 connection error.

#### T12.4 Wait-ready signal handling

- **File**: `crates/rspm-daemon/src/supervisor/lifecycle.rs`.
- **Flow**:
  ```
  app sends "ready" via stdin pipe (IPC ringan) OR via env var RSPM_READY_FIFO
  daemon listens for "ready" string from child stdout / control fd
  ```
- **DoD**: child yang kirim ready dianggap online; yang tidak setelah listen_timeout → tetap online (kompat PM2).

---

### Phase 13 — Save / Resurrect / Startup

#### T13.1 CLI: save / dump

- **File**: `crates/rspm-cli/src/commands/save.rs`, `dump.rs`.
- **Flow**: panggil `SaveProcessList`.
- **DoD**: file `dump.rspm` ada.

#### T13.2 CLI: resurrect

- **File**: `crates/rspm-cli/src/commands/resurrect.rs`.
- **Flow**: panggil `Resurrect`.
- **DoD**: setelah kill + resurrect, semua jalan.

#### T13.3 Init system detection

- **File**: `crates/rspm-startup/src/detect.rs`.
- **Flow**: cek `/run/systemd/system` (systemd), `/sbin/openrc` (openrc), `/etc/init.d/.legacy` (sysv).
- **DoD**: 3 distro matrix tested.

#### T13.4 Generator template (tera)

- **File**: `crates/rspm-startup/src/generator.rs` + `templates/*.tera`.
- **Flow**:
  ```
  context = { user, home, rspm_bin_path, work_dir }
  match init { Systemd => render systemd.tera, ... }
  ```
- **PM2 Ref**: `pm2/lib/templates/init-scripts/*.tpl`.
- **DoD**: unit file `systemctl daemon-reload` accept.

#### T13.5 CLI: startup install

- **File**: `crates/rspm-cli/src/commands/startup.rs`.
- **Flow**:
  ```
  init = detect or override --platform
  unit_text = generate(...)
  if running as root: write file, enable service
  else: print "run as sudo: ..."
  ```
- **DoD**: systemd service jalan saat reboot (manual verify).

#### T13.6 CLI: unstartup (remove)

- **Flow**: disable + remove unit file.
- **DoD**: bersih.

---

### Phase 14 — TUI Dashboard

#### T14.1 Ratatui app skeleton

- **File**: `crates/rspm-dashboard/src/app.rs`, `ui.rs`.
- **Flow**: table widget kiri (list process), panel kanan (logs tail + metrics chart).
- **PM2 Ref**: `pm2/lib/API/Monit.js`.
- **DoD**: 5 fps render OK.

#### T14.2 Keymap (q quit, r restart, s stop, d delete, ↑↓ navigate)

- **File**: `crates/rspm-dashboard/src/input.rs`.
- **DoD**: tombol berfungsi.

#### T14.3 CLI: monit

- **File**: `crates/rspm-cli/src/commands/monit.rs`.
- **Flow**: launch dashboard subscribe bus + poll list.
- **DoD**: berjalan.

---

### Phase 15 — Modules

#### T15.1 Install module via npm

- **File**: `crates/rspm-modules/src/installer.rs`.
- **Flow**: `npm install --prefix $RSPM_HOME/modules/<name> <package>`, baca package.json, deteksi entrypoint.
- **PM2 Ref**: `pm2/lib/API/Modules/Modularizer.js`.
- **DoD**: `rspm install pm2-logrotate` (sebagai test pakai sample dummy npm) sukses.

#### T15.2 Module sebagai app

- **Flow**: setelah install, generate AppConfig dari package.json `apps`, panggil start_app.
- **DoD**: module jalan sebagai process.

#### T15.3 Uninstall module

- **DoD**: stop + delete + rm dir.

---

### Phase 16 — Deploy (SSH)

#### T16.1 Deploy config schema

- **File**: `crates/rspm-deploy/src/lib.rs`.
- **Flow**: struct mirror PM2 `deploy: { production: { user, host, repo, path, pre-deploy, post-deploy } }`.
- **DoD**: parse sukses.

#### T16.2 SSH connect (russh)

- **File**: `crates/rspm-deploy/src/ssh.rs`.
- **DoD**: bisa run remote `uname`.

#### T16.3 Deploy flow: setup / deploy

- **File**: `crates/rspm-deploy/src/lib.rs`.
- **Flow**:
  ```
  setup: ssh + git clone repo path
  deploy: ssh + cd path + git fetch + checkout ref + pre-deploy hook + npm install + post-deploy hook + rspm reload ecosystem
  ```
- **PM2 Ref**: `pm2-deploy` package.
- **DoD**: deploy ke localhost test container OK.

---

### Phase 17 — HTTP API (Optional Feature)

#### T17.1 axum server

- **File**: `crates/rspm-http/src/server.rs`.
- **Flow**: routes: GET `/processes`, POST `/start`, POST `/stop/:id`, GET `/logs/:id` (SSE).
- **DoD**: curl test OK.

#### T17.2 Feature flag di Cargo.toml

- **Flow**: `[features] http = ["dep:axum"]`. Default off.
- **DoD**: build tanpa feature lebih kecil.

---

### Phase 18 — Cross-platform Polish (post-v0.0.1 stub)

#### T18.1 macOS launchd template (stub)

- **File**: `crates/rspm-startup/src/templates/launchd.tera`.
- **DoD**: template file exists, behind `#[cfg(target_os="macos")]`.

#### T18.2 Windows service stub (post v0.0.1)

- **DoD**: docs.md mentioning future.

---

### Phase 19 — Quality / Release

#### T19.1 Integration test suite (tests/)

- **File**: `tests/e2e/*.rs`.
- **Flow**: spawn daemon process, run CLI commands, assert states.
- **DoD**: 30+ test, jalan di CI Linux.

#### T19.2 Migration guide

- **File**: `docs/migration-from-pm2.md`.
- **Content**: command equivalence table, config differences, known incompatibilities.
- **DoD**: rendered markdown OK.

#### T19.3 cargo-deny enforce

- **File**: `deny.toml`.
- **DoD**: CI gate.

#### T19.4 Release: cargo build --release + checksum

- **File**: `.github/workflows/release.yml`.
- **Flow**: tag push → build → upload artifact + sha256.
- **DoD**: artifact downloadable.

#### T19.5 Crates.io publish workflow

- **Flow**: publish in dep order: core → protocol → ipc → config → ... → cli.
- **DoD**: dry-run sukses.

---

## 9. Verification Plan (Per Phase)

| Phase | Verifikasi                                                                             |
| ----- | -------------------------------------------------------------------------------------- |
| 0     | `cargo build --workspace` 0 error. `cargo clippy -- -D warnings` 0.                    |
| 1     | Unit test `rspm-core` ≥ 80% coverage (types, paths, defaults).                         |
| 2     | Round-trip serialize semua Request/Response/Event variant.                             |
| 3     | Integration test: server-client UDS communication 1000x request tanpa drop.            |
| 4     | Golden fixture: parse PM2 `examples/ecosystem.config.js` resmi → struct expected.      |
| 5     | E2E: spawn daemon, start `node index.js`, list shows online, kill daemon → child mati. |
| 6     | E2E: stop/restart/delete/scale tested manual + automated.                              |
| 7     | CLI smoke test: `rspm start app.js && rspm list && rspm stop 0 && rspm delete all`.    |
| 8     | Long-running test: cron restart trigger setiap menit verified 5 menit.                 |
| 9     | Compare `rspm monit` output dengan `htop` ±5% accurate.                                |
| 10    | Touch file → child restart dalam ≤ 500ms (debounce).                                   |
| 11    | Generate 10 MB log → rotate ≤ 3 archive.                                               |
| 12    | wrk load test selama `rspm reload` → 0 dropped connection.                             |
| 13    | `rspm startup systemd && reboot` → service jalan otomatis.                             |
| 14    | `rspm monit` interactive: 60 fps stress test.                                          |
| 15    | Install module, restart daemon, module otomatis jalan.                                 |
| 16    | Deploy ke container Ubuntu, verify app running.                                        |
| 17    | `curl localhost:9615/processes` return valid JSON.                                     |
| 18    | Linux-only build matrix CI.                                                            |
| 19    | All e2e green, release artifact diunduh + jalan di Ubuntu fresh VM.                    |

### 9.1 PM2 Parity Test Matrix

Buat skrip `tests/parity/` yang menjalankan command sama di PM2 dan rspm, lalu compare JSON output `jlist`. Toleransi field: skip `_id` / timestamps.

| Command                                | PM2 | rspm | Expected                            |
| -------------------------------------- | --- | ---- | ----------------------------------- |
| `start app.js`                         | ✓   | ✓    | jlist[0].pm2_env.status == "online" |
| `start --name X`                       | ✓   | ✓    | name == "X"                         |
| `start ecosystem.config.js --env prod` | ✓   | ✓    | env.NODE_ENV == "production"        |
| `restart 0`                            | ✓   | ✓    | restart_time++                      |
| `reload 0`                             | ✓   | ✓    | uptime baru, 0 downtime             |
| `scale app 4`                          | ✓   | ✓    | jlist length == 4                   |
| `stop all`                             | ✓   | ✓    | semua "stopped"                     |
| `delete all`                           | ✓   | ✓    | jlist == []                         |
| `save && kill && resurrect`            | ✓   | ✓    | state restored                      |

---

## 10. Critical Files Cheatsheet

File yang AI vibe coding harus baca lebih dulu sebelum task tertentu:

| Task Group       | File PM2 Mutlak                                                              | Cara Baca         |
| ---------------- | ---------------------------------------------------------------------------- | ----------------- |
| T1.\* (types)    | `pm2/constants.js`, `pm2/paths.js`, `pm2/lib/API/schema.json`                | full read         |
| T2.\* (protocol) | `pm2/lib/God/ActionMethods.js`, `pm2/lib/God.js:200-400`                     | skim method names |
| T3.\* (IPC)      | `pm2/lib/Daemon.js:200-300`, `pm2/lib/Client.js:200-400`                     | skim              |
| T4.\* (config)   | `pm2/lib/Common.js:105-572`, `pm2/lib/API/schema.json`                       | full read         |
| T5.\* (daemon)   | `pm2/lib/Daemon.js`, `pm2/lib/God.js`, `pm2/lib/God/ForkMode.js`             | full read         |
| T6.\* (handlers) | `pm2/lib/God/ActionMethods.js` (~910 lines)                                  | by method         |
| T7.\* (CLI)      | `pm2/lib/binaries/CLI.js` (~1078 lines)                                      | by subcommand     |
| T8.\* (worker)   | `pm2/lib/Worker.js`                                                          | full read         |
| T10.\* (watch)   | `pm2/lib/Watcher.js`                                                         | full read         |
| T11.\* (logs)    | `pm2/lib/Utility.js`, `pm2/lib/API/Log.js`                                   | full read         |
| T12.\* (cluster) | `pm2/lib/God/ClusterMode.js`, `pm2/lib/God/Reload.js`                        | full read         |
| T13.\* (startup) | `pm2/lib/API/Startup.js`, `pm2/lib/templates/init-scripts/*.tpl`             | full read         |
| T14.\* (monit)   | `pm2/lib/API/Monit.js`                                                       | full read         |
| T15.\* (modules) | `pm2/lib/API/Modules/` (semua)                                               | skim              |
| T16.\* (deploy)  | `pm2/lib/API/Deploy.js`, `node_modules/pm2-deploy/lib/deploy.js` (kalau ada) | skim              |

---

## 11. PM2 Source Reference Index

Index cepat (offset baris kira-kira berdasarkan PM2 v5.x):

```
pm2/
├── constants.js               # KILL_TIMEOUT 1600, WORKER_INTERVAL 30000, dll
├── paths.js                   # PM2_HOME path
├── lib/
│   ├── Common.js              # 105-572  prepareAppConf, resolveEnvironment
│   ├── Daemon.js              # 1-456    entry, processStateHandler, RPC bind
│   ├── God.js                 # 1-633    singleton, executeApp, handleExit
│   ├── Client.js              # 1-777    launchDaemon, RPC client
│   ├── Watcher.js             # chokidar wrapper
│   ├── Worker.js              # 1-170    cron, mem check
│   ├── Utility.js             # timestamp helper
│   ├── God/
│   │   ├── ActionMethods.js   # 1-910    ALL RPC handlers
│   │   ├── ForkMode.js        # 1-303    spawn child
│   │   ├── ClusterMode.js     # cluster.fork
│   │   ├── Reload.js          # 1-241    soft/hard reload
│   │   └── SystemData.js      # /proc sampler
│   ├── API/
│   │   ├── schema.json        # config field schema (CANONICAL)
│   │   ├── Log.js             # tail UI
│   │   ├── Monit.js           # TUI
│   │   ├── Startup.js         # init script generator
│   │   ├── Deploy.js          # SSH deploy
│   │   └── Modules/           # module install
│   ├── binaries/
│   │   └── CLI.js             # 1-1078   semua subcommand commander.js
│   └── templates/
│       └── init-scripts/      # *.tpl untuk systemd/openrc/sysv/launchd
```

---

## 12. Execution Notes untuk AI Vibe Coding

### 12.1 Eksekusi Berurutan

- Kerjakan task dalam urutan T0._ → T1._ → ... → T19.\*.
- **Jangan lompat phase** kecuali ada alasan teknis kuat (catat di commit message).
- Dalam 1 phase, urutan task **boleh** disesuaikan kalau ada dependency teknis.

### 12.2 Branch & PR Strategy

- 1 task = 1 branch = 1 PR.
- Branch name: `task/T<phase>.<idx>-<kebab-summary>`.
- Contoh: `task/T1.3-rspm-home-path-resolver`.
- PR title: `feat(rspm-core): T1.3 RspmHome path resolver`.

### 12.3 Tooling Checklist per PR

```
[ ] cargo fmt --all
[ ] cargo clippy --all-targets --all-features -- -D warnings
[ ] cargo test --workspace
[ ] cargo deny check (jika tambah dep)
[ ] cargo doc --no-deps (jika tambah pub API)
```

### 12.4 Saat Behavior PM2 Ambigu

1. Tulis test case yang menggambarkan behavior expected.
2. Jalankan command setara di PM2 (di folder `pm2/` ada test suite-nya juga).
3. Replikasi behavior 1:1 di Rust.
4. Catat divergence (kalau ada) di `docs/migration-from-pm2.md`.

### 12.5 Saat Field Schema PM2 Tidak Terdokumentasi

- `pm2/lib/API/schema.json` = canonical source. Kalau field ada di sini tapi tidak ada di docs PM2, **tetap dukung**.
- Field PM2 yang DEPRECATED jangan diimplement (catat di section "Skipped" migration doc).

### 12.6 Threading Model

- Daemon: 1 tokio runtime multi-thread, default worker_threads = num_cpus.
- 1 task tokio per accept loop (rpc & pub), 1 task per child process (wait), 1 task per child log stream (stdout, stderr), 1 task worker tick.
- State God dilindungi `tokio::sync::Mutex` (bukan `std::sync::Mutex` — kita pakai await di dalam).

### 12.7 Error UX

- Error message yang muncul ke user CLI **wajib actionable**: "tidak ditemukan, coba `rspm list`" lebih baik daripada "process not found".
- Format: `error: <kategori>: <detail>\nhelp: <saran>`.

### 12.8 Logging Internal

- `tracing` dengan span per request. Level INFO default, DEBUG via `RUST_LOG=rspm=debug`.
- Daemon log file: `$RSPM_HOME/pm2.log` (nama compat).

### 12.9 Stabilitas Wire Protocol

- `PROTOCOL_VERSION = 1` untuk semua v0.x. Saat breaking change protokol → bump.
- Client/daemon mismatch → reject di handshake, error jelas suggest `rspm update`.

### 12.10 Catatan Khusus untuk MVP v0.0.1

- Phase 0–7 = MUST untuk v0.0.1 release.
- Phase 8–11 = MUST untuk v0.1.0.
- Phase 12 (cluster) = penting untuk D4 → boleh di v0.1.0 atau v0.2.0.
- Phase 13–19 = bertahap.

### 12.11 Yang TIDAK Akan Diimplementasi di v0.x

- pm2.io / Keymetrics integration.
- `pm2 plus` features.
- Web monit UI (browser).
- Native Windows service (post 1.0).

---

**Akhir Plan.** Plan ini siap dipakai vibe coding AI untuk implementasi rspm v0.0.1.
