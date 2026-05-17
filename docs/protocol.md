# RSPM Wire Protocol

Versi: **`PROTOCOL_VERSION = 1`** (`crates/rspm-protocol/src/version.rs`).

## Frame Format

Setiap frame = 4 byte length prefix (big-endian u32) + payload JSON.

```
+--------+--------------------+
| u32 BE | JSON payload bytes |
| length | (serde_json)       |
+--------+--------------------+
```

- `length` adalah byte count payload (tidak termasuk 4 byte header).
- Maksimum frame: **16 MiB** (`MAX_FRAME_BYTES`). Lebih besar → server tutup koneksi dengan error.
- Encoding payload: UTF-8 JSON via `serde_json`.

Implementasi: `crates/rspm-protocol/src/frame.rs` (`read_frame` / `write_frame`).

## Socket Layout

| Path | Arah | Isi |
|------|------|-----|
| `$RSPM_HOME/rpc.sock` | client → daemon (request/reply) | `Request` → `Response` |
| `$RSPM_HOME/pub.sock` | daemon → subscriber (broadcast) | stream `Event` |

Permission: socket file dibuat dengan default umask. Daemon hapus stale socket sebelum bind.

## Request

JSON tag: `method`, content: `params`. Snake-case.

```jsonc
// Ping
{ "method": "ping" }

// Start app
{ "method": "start", "params": { "app": { /* AppConfig */ } } }

// Stop by selector
{ "method": "stop", "params": { "selector": "all" } }
{ "method": "stop", "params": { "selector": { "id": 3 } } }
{ "method": "stop", "params": { "selector": { "name": "api" } } }
```

| Method | Params | Notes |
|--------|--------|-------|
| `ping` | – | Health check |
| `get_version` | – | Returns daemon binary version |
| `list` | – | Lists all `ProcessInfo` |
| `start` | `{ app: AppConfig }` | Spawn satu app (multi instance bila `instances > 1`) |
| `stop` | `{ selector }` | SIGINT → `kill_timeout_ms` → SIGKILL |
| `restart` | `{ selector }` | Stop + spawn ulang, increment `restart_time` |
| `reload` | `{ selector }` | Cluster: rolling reload SO_REUSEPORT. Fork: fallback ke restart. |
| `delete` | `{ selector }` | Stop + remove dari registry |
| `logs` | `{ selector?, lines }` | Tail N baris dari per-stream log file |
| `save` | – | Tulis daftar `AppConfig` ke `dump.rspm` (atomic via temp + rename) |
| `resurrect` | – | Baca `dump.rspm` → start setiap app (skip kalau sudah jalan) |
| `send_signal` | `{ selector, signal }` | Kirim Unix signal: `SIGINT/TERM/HUP/USR1/USR2/KILL` |
| `kill_daemon` | – | Stop semua proses + shutdown daemon |

`Selector` = `"all"` | `{ "id": u32 }` | `{ "name": "..." }`.

## Response

JSON tag: `status`, content: `data`.

```jsonc
{ "status": "ack", "data": { "message": "ok" } }
{ "status": "pong", "data": { "msg": "pong" } }
{ "status": "version", "data": { "version": "0.0.1" } }
{ "status": "process_list", "data": { "processes": [ /* ProcessInfo */ ] } }
{ "status": "started", "data": { "processes": [ /* ProcessInfo */ ] } }
{ "status": "process", "data": { "process": { /* ProcessInfo */ } } }
{ "status": "logs", "data": { "lines": ["[api] [out] ready"] } }
{ "status": "error", "data": { "message": "process not found" } }
```

Client wajib treat `status: "error"` sebagai failure. `Response::into_result()` di Rust API otomatis konversi ke `RspmError::Daemon(message)`.

## Event (pub.sock)

JSON tag: `event`, content: `data`. Stream tak-terbatas selama koneksi hidup.

```jsonc
{ "event": "process_online", "data": { "process": { /* ProcessInfo */ } } }
{ "event": "process_exit",   "data": { "pm_id": 0, "code": 143 } }
{ "event": "process_exit",   "data": { "pm_id": 0, "code": null } }
{ "event": "log", "data": {
    "pm_id": 0,
    "name": "api",
    "stream": "out",
    "data": "ready",
    "at": "2026-05-17T18:10:00.123Z"
  }
}
{ "event": "log", "data": { "stream": "err", ... } }
{ "event": "process_msg", "data": { "pm_id": 1, "payload": {"ready": true} } }
{ "event": "system_warn", "data": { "message": "fd exhaustion" } }
```

Subscriber connection (Rust):
```rust
let mut subscriber = rspm_ipc::EventSubscriber::connect(&home.pub_socket()).await?;
while let Some(event) = subscriber.next_event().await? {
    // handle event
}
```

## AppConfig (Subset Field)

Lihat `crates/rspm-core/src/types/app.rs` untuk daftar lengkap. Field yang sering dipakai client:

| Field | Type | Default | PM2 alias |
|-------|------|---------|-----------|
| `name` | `String` | derived from script stem | `name` |
| `script` | `PathBuf` | required | `script` |
| `args` | `Vec<String>` | `[]` | `args` |
| `cwd` | `Option<PathBuf>` | inherit daemon | `cwd` |
| `execution_mode` | `"fork_mode" \| "cluster_mode"` | `fork_mode` | `exec_mode` |
| `instances` | `u32` \| `"max"` \| `"-1"` | `1` | `instances` |
| `max_memory_restart` | `Option<String>` (e.g. `"200M"`) | `None` | `max_memory_restart` |
| `auto_restart` | `bool` | `true` | `autorestart` |
| `watch` | `bool` \| `[String]` | `false` | `watch` |
| `ignore_watch` | `Vec<String>` | `[]` | `ignore_watch` |
| `kill_timeout_ms` | `u64` | `1600` | `kill_timeout` |
| `min_uptime_ms` | `u64` | `1000` | `min_uptime` |
| `max_restarts` | `u32` | `16` | `max_restarts` |
| `restart_delay_ms` | `u64` | `0` | `restart_delay` |
| `exp_backoff_restart_delay_ms` | `Option<u64>` | `None` | `exp_backoff_restart_delay` |
| `stop_exit_codes` | `Vec<i32>` | `[]` | `stop_exit_codes` |
| `env` | `BTreeMap<String,String>` | `{}` | `env` |
| `error_file` / `out_file` / `combined_file` | `Option<PathBuf>` | derived | `error_file` / `out_file` / `log_file` |
| `log_date_format` | `Option<String>` (strftime) | `None` | `log_date_format` |
| `merge_logs` | `bool` | `false` | `merge_logs` |
| `prefix_timestamp` | `bool` | `false` | `time` |
| `cron_restart` | `Option<String>` | `None` | `cron_restart` |
| `interpreter` | `Option<PathBuf>` | auto-detect (node/python) | `interpreter` |
| `interpreter_args` | `Vec<String>` | `[]` | `node_args` |
| `instance_var` | `String` | `"NODE_APP_INSTANCE"` | `instance_var` |
| `wait_ready` | `bool` | `false` | `wait_ready` |
| `listen_timeout_ms` | `u64` | `8000` | `listen_timeout` |

## Handshake (Reserved)

Versi 1 belum mewajibkan handshake eksplisit. Saat protokol berubah, request pertama harus berupa `{ method: "handshake", params: { protocol_version: 1, client_version: "..." } }` dan daemon menjawab `{ status: "handshake_ack", data: { protocol_version: 1, daemon_version: "..." } }`. Mismatch → tutup koneksi dengan `Error { message: "protocol mismatch, run rspm update" }`.

Implementasi placeholder: `crates/rspm-ipc/src/handshake.rs`.

## Backwards Compatibility

- `PROTOCOL_VERSION = 1` dipertahankan untuk seluruh seri `0.x`. Bumping = breaking change, harus dicatat di `CHANGELOG`.
- Field baru di `AppConfig` selalu `#[serde(default)]` agar dump file lama tetap bisa di-resurrect.
- Variant baru di `Request`/`Response`/`Event` aman ditambah; client lama akan terima `Error` untuk method baru yang tidak di-handle daemon lama.
