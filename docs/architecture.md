# RSPM Architecture

> Cara kerja `rspm` (Rust Process Manager). Versi 0.0.1.

## Tujuan

`rspm` adalah supervisor proses long-running yang functional-equivalent dengan PM2 (Node.js) tapi ditulis dalam Rust. Daemon tunggal (`rspm-daemon`) mengelola seluruh proses anak; CLI (`rspm`) berkomunikasi via Unix Domain Socket.

## Flow Tingkat Tinggi

```
+--------------+    1. UDS request frame     +-------------+    spawn/signal     +--------------+
| rspm CLI     | -------------------------> | rspm-daemon | -----------------> | child app    |
| (rspm-cli)   | <------------------------- | (rspm-daemon)|                     | (Node/Go/Py)|
+--------------+    2. UDS response frame    +-------------+ <----- stdout/stderr +--------------+
                                                |
                                                | 3. publish Event
                                                v
                                          +-----------+
                                          |  pub.sock | <--- rspm logs --follow
                                          | broadcast |
                                          +-----------+
```

1. **CLI** parse argument dengan `clap` → bangun `Request` → kirim ke daemon via `rpc.sock`.
2. **Daemon** terima `Request`, dispatch ke handler `God`, balas `Response`.
3. **Event** (ProcessOnline, ProcessExit, Log) dipublish ke `PubSubBus`, di-forward ke subscriber lewat `pub.sock`.

## Lapisan Crate

| Crate | Lapisan | Tanggung Jawab |
|-------|---------|----------------|
| `rspm-core` | Foundation | Tipe data (`AppConfig`, `ProcessInfo`, `ProcessStatus`), error, defaults, path resolver (`RspmHome`) |
| `rspm-protocol` | Wire | `Request`, `Response`, `Event` enum + length-prefixed JSON frame codec |
| `rspm-ipc` | Transport | `IpcServer`/`IpcClient` (UDS request/reply) + `PubSubBus`/`EventSubscriber` |
| `rspm-config` | Input | Loader TOML/YAML/JSON/`ecosystem.config.js` (boa), normalisasi, env expansion |
| `rspm-logs` | Output | `LogWriter` (timestamp prefix, merge_logs), `Rotator` (size-based archives), `tail_file` |
| `rspm-monitor` | Sampler | `Sampler` (sysinfo) + `Aggregator` (rolling window CPU/MEM) |
| `rspm-watcher` | Side-effect | `notify`-based file watch + glob matcher + debouncer |
| `rspm-cluster` | Network | `SO_REUSEPORT` socket binding helper |
| `rspm-startup` | Bootstrap | Generate systemd/openrc/sysv unit (tera) + install/uninstall |
| `rspm-daemon` | Core | `God` supervisor + handler dispatch + worker tick + lifecycle |
| `rspm-client` | Lib | `RspmClient` high-level API + daemon auto-launcher |
| `rspm-cli` | Binary | `rspm` entry point (clap → client) |

Dependency graph:

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
   └── rspm-startup ───┘
```

`rspm-core` adalah satu-satunya crate yang **tidak boleh** import crate workspace lain.

## God Supervisor

`God` = singleton state daemon. Komponen utama:

```rust
struct God {
    home: RspmHome,                                          // dirs + sockets
    next_id: ProcessId,                                      // monotonic counter
    processes: BTreeMap<ProcessId, ManagedProcess>,          // registry
    sampler: Sampler,                                        // CPU/MEM polling
    aggregator: Aggregator,                                  // rolling window
    cron_next: HashMap<ProcessId, DateTime<Utc>>,            // cron schedule
    restart_tx/rx: mpsc<ProcessId>,                          // watcher → restart
    bus: PubSubBus,                                          // event broadcast
}

struct ManagedProcess {
    app: AppConfig,                                          // user config
    info: ProcessInfo,                                       // public state
    child: Option<tokio::process::Child>,                    // OS process
    instance_index: u32,                                     // cluster instance
    watcher: Option<JoinHandle<()>>,                         // file watcher task
    log_tasks: Vec<JoinHandle<()>>,                          // stdout/stderr forwarders
    prev_restart_delay_ms: u64,                              // exp backoff state
    next_restart_at: Option<tokio::time::Instant>,           // scheduled restart
}
```

State seluruhnya dilindungi `tokio::sync::Mutex` (lihat `crates/rspm-daemon/src/lib.rs::run`). RPC handler ambil lock sebentar saja sebelum memanggil method God.

## Lifecycle Proses Anak

```
        +-----------+
        | LAUNCHING |  (start_app / restart_id mulai spawn)
        +-----+-----+
              |
       spawn OK + bus publish ProcessOnline
              v
        +-----------+
        |  ONLINE   | <----- worker_tick sample CPU/MEM
        +-----+-----+
              |
       child exit OR stop_id called
              v
        +-----------+
        | STOPPING  |  (SIGINT + tunggu kill_timeout_ms)
        +-----+-----+
              |
              v
        +-----------+      auto_restart=true & exit non-zero & not in stop_exit_codes
        |  STOPPED  | --------------------> WAITING (delay) -----> LAUNCHING (loop)
        +-----------+      else stays Stopped
              |
              | unstable_restarts >= max_restarts
              v
        +-----------+
        |  ERRORED  |
        +-----------+
```

## Restart Policy

Mirror PM2 `lib/God.js:handleExit`:

1. Hitung `uptime = now - pm_uptime`.
2. Cek `stop_exit_codes`: kalau code ada di list → mark `Stopped`, jangan restart.
3. Cek `auto_restart`: false → mark `Stopped`.
4. Kalau `uptime < min_uptime_ms` → `unstable_restarts += 1`.
5. Kalau `unstable_restarts >= max_restarts` atau `restart_time >= max_restarts` → `Errored`, stop loop.
6. Hitung delay:
   - `exp_backoff_restart_delay_ms` set? pakai `next_exp_backoff(prev, base)` (1.5× cap 15s).
   - else pakai `restart_delay_ms` (default 0).
7. Set `next_restart_at = now + delay`, mark `Waiting`.
8. Worker tick + RPC list akan trigger spawn ulang saat deadline tercapai.
9. Worker tick reset `prev_restart_delay_ms = 0` saat uptime > 30s (PM2 parity).

## Log Capture

Sebelum fix: daemon pipe stdout/stderr child langsung ke file plain (tidak ada rotasi, tidak ada bus event).

Sekarang:

```
child.stdout (piped) ─► tokio task per stream
                       ├─► LogWriter.write_line  → app-{name}-out.log (rotated)
                       └─► bus.publish(Event::Log)  → pub.sock subscribers
```

`LogWriter` urusi: timestamp prefix (`prefix_timestamp` + `log_date_format`), merge label (`merge_logs`), dan rotation (default 10MB × 10 archives).

## IPC Wire Format

```
+--------+--------------------+
| u32 BE | JSON payload bytes |
| length | (serde_json)       |
+--------+--------------------+
```

`MAX_FRAME_BYTES = 16 MiB`. `PROTOCOL_VERSION = 1` untuk seluruh v0.x. Mismatch ditolak di handshake.

Detail message: lihat `protocol.md`.

## Worker Tick

Default interval: 30s (`WORKER_INTERVAL_MS`). Setiap tick:

1. `refresh_and_autorestart` — poll exit status, schedule restart sesuai policy.
2. `reset_stable_backoff` — reset `prev_restart_delay_ms` untuk proses stabil >30s.
3. `sample_metrics` — `sysinfo` ambil CPU%/MEM, smoothing rolling window (default 10 sample).
4. `check_memory_limits` — restart proses yang melebihi `max_memory_restart`.
5. `run_cron_restarts` — restart proses yang cron-spec-nya jatuh tempo.

## File Layout (`$RSPM_HOME`)

```
$RSPM_HOME (default ~/.rspm/)
├── pids/              # per-app pid + ready sentinel
├── logs/              # per-app stdout/stderr (rotated)
├── modules/           # installed modules (post v0.0.1)
├── pm2.log            # daemon tracing log (nama PM2-compat)
├── pm2.pid            # daemon pid lock
├── rpc.sock           # IPC request/reply
├── pub.sock           # IPC pub/sub events
├── dump.rspm          # persisted process list (JSON)
└── dump.rspm.bak      # backup sebelum overwrite
```

## Referensi PM2 Source

| Konsep RSPM | PM2 file |
|-------------|----------|
| `God` singleton | `pm2/lib/God.js` |
| Fork mode spawn | `pm2/lib/God/ForkMode.js` |
| Cluster mode | `pm2/lib/God/ClusterMode.js` |
| Soft reload | `pm2/lib/God/Reload.js` |
| RPC handlers | `pm2/lib/God/ActionMethods.js` |
| Daemon entry | `pm2/lib/Daemon.js` |
| Client launcher | `pm2/lib/Client.js` |
| Config schema canonical | `pm2/lib/API/schema.json` |
| Config normalization | `pm2/lib/Common.js:105-572` |
| Watcher | `pm2/lib/Watcher.js` |
| Worker tick | `pm2/lib/Worker.js` |
| CLI | `pm2/lib/binaries/CLI.js` |
| Constants | `pm2/constants.js`, `pm2/paths.js` |
