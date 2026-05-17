# Migration from PM2 to RSPM

> Target audience: pengguna PM2 yang ingin pindah ke `rspm`. Versi `0.0.1`.

## Filosofi Migrasi

`rspm` dirancang sebagai **functional-equivalent**, bukan binary-equivalent. Goals:
- Command CLI familiar (`start`/`stop`/`restart`/`list`/`logs`/`save`/`resurrect`).
- Field config sama (snake_case PM2) — `ecosystem.config.js` lama bisa langsung dipakai.
- `$RSPM_HOME` mirror struktur `~/.pm2/` (nama `pm2.log`/`pm2.pid` dipertahankan untuk tooling lama).

`rspm` **tidak** kompatibel binary-level dengan `pm2`:
- Wire protocol berbeda (Rust-native JSON frame vs `pm2-axon`).
- Dump file format JSON `dump.rspm` (bukan `dump.pm2` Node serialization).
- Tidak bisa pakai daemon PM2 lama dengan CLI rspm dan sebaliknya.

## Command Equivalence

| PM2 | RSPM | Status | Catatan |
|-----|------|--------|---------|
| `pm2 start app.js` | `rspm start app.js` | ✅ | Identik |
| `pm2 start ecosystem.config.js` | `rspm start ecosystem.config.js` | ✅ | Parsed via `boa_engine` |
| `pm2 start app.js --name api` | `rspm start app.js --name api` | ✅ | |
| `pm2 start app.js -i 4` | `rspm start app.js -i 4` | ✅ | Cluster via SO_REUSEPORT, bukan Node cluster module |
| `pm2 stop <id\|name\|all>` | `rspm stop <id\|name\|all>` | ✅ | |
| `pm2 restart <id\|name\|all>` | `rspm restart <id\|name\|all>` | ✅ | |
| `pm2 reload <id\|name>` | `rspm reload <id\|name>` | 🟡 | Cluster: zero-downtime via SO_REUSEPORT + ready sentinel. Fork: fallback restart. |
| `pm2 delete <id\|name\|all>` | `rspm delete <id\|name\|all>` | ✅ | |
| `pm2 list` / `pm2 ls` | `rspm list` / `rspm ls` | ✅ | Default subcommand |
| `pm2 jlist` | `rspm jlist` | ✅ | JSON array |
| `pm2 prettylist` | `rspm prettylist` | ✅ | |
| `pm2 logs [name] --lines N` | `rspm logs [name] --lines N` | 🟡 | Tail-only di CLI; subscribe real-time via `EventSubscriber` API |
| `pm2 logs --follow` | (via `rspm_ipc::EventSubscriber`) | 🟡 | Belum ada flag CLI; subscriber programmatic sudah ada |
| `pm2 save` | `rspm save` | ✅ | Dump JSON ke `dump.rspm` |
| `pm2 resurrect` | `rspm resurrect` | ✅ | |
| `pm2 ping` | `rspm ping` | ✅ | |
| `pm2 kill` | `rspm kill` | ✅ | Stop semua + shutdown daemon |
| `pm2 sendSignal <sig> <name>` | `rspm send-signal <sig> <name>` | ✅ | |
| `pm2 startup` | `rspm startup [--platform systemd]` | ✅ | systemd/openrc/sysv |
| `pm2 unstartup` | `rspm unstartup` | ✅ | |
| `pm2 monit` | `rspm monit` | ❌ | TUI belum (Phase 14, post-v0.1.0) |
| `pm2 describe <id>` | `rspm prettylist` | 🟡 | Belum dipisah command-nya |
| `pm2 deploy` | – | ❌ | Phase 16 (post-v0.1.0) |
| `pm2 install <module>` | – | ❌ | Phase 15 (post-v0.1.0) |
| `pm2 module:install/uninstall` | – | ❌ | Phase 15 |
| `pm2 web` / HTTP API | – | ❌ | Phase 17 (optional feature) |
| `pm2 flush` | – | ❌ | Belum diimplementasi |
| `pm2 reloadLogs` | – | ❌ | Belum diimplementasi (rotator otomatis 10MB) |
| `pm2 scale app N` | – | 🟡 | Tidak ada command langsung; pakai `restart` dengan config `instances: N` baru |

Legend: ✅ = full parity, 🟡 = partial / behavior berbeda, ❌ = belum diimplementasi.

## Config Field Equivalence

Field di `ecosystem.config.js` / `apps.json` / `apps.toml`:

| PM2 Field | RSPM Internal | Aliases Diterima | Catatan |
|-----------|---------------|------------------|---------|
| `name` | `name` | – | |
| `script` | `script` | `exec` | |
| `args` | `args` | – | String atau array |
| `cwd` | `cwd` | – | |
| `exec_mode` | `execution_mode` | `exec_mode` | `"fork"`/`"cluster"` |
| `instances` | `instances` | – | `1` / `"max"` / `-1` / `N` |
| `max_memory_restart` | `max_memory_restart` | – | `"200M"`, `"1G"`, dll |
| `autorestart` | `auto_restart` | `autorestart` | |
| `watch` | `watch` | – | `bool` atau array path |
| `ignore_watch` | `ignore_watch` | – | |
| `kill_timeout` | `kill_timeout_ms` | `kill_timeout` | |
| `min_uptime` | `min_uptime_ms` | `min_uptime` | |
| `max_restarts` | `max_restarts` | – | |
| `restart_delay` | `restart_delay_ms` | `restart_delay` | Baru di rspm |
| `exp_backoff_restart_delay` | `exp_backoff_restart_delay_ms` | – | 1.5× cap 15s |
| `stop_exit_codes` | `stop_exit_codes` | – | Array i32 |
| `env` | `env` | – | |
| `env_<NAME>` | `env_overrides[NAME]` | – | Merge via `--env` (TBD) |
| `error_file` | `error_file` | `err`, `err_file`, `err_log` | |
| `out_file` | `out_file` | `out`, `output`, `out_log` | |
| `log_file` | `combined_file` | `log` | |
| `log_date_format` | `log_date_format` | – | strftime |
| `merge_logs` | `merge_logs` | – | |
| `time` | `prefix_timestamp` | `time` | |
| `cron_restart` | `cron_restart` | – | cron spec |
| `interpreter` | `interpreter` | `exec_interpreter` | `"none"` = no interpreter |
| `node_args` | `interpreter_args` | `node_args`, `interpreterArgs` | |
| `instance_var` | `instance_var` | – | default `NODE_APP_INSTANCE` |
| `wait_ready` | `wait_ready` | – | App harus tulis file `$RSPM_READY_FILE` |
| `listen_timeout` | `listen_timeout_ms` | `listen_timeout` | |

## Behavior Differences

### Cluster Mode

PM2 pakai **Node.js cluster module** yang fork child pakai IPC channel khusus Node. Hanya jalan untuk Node app.

RSPM pakai **`SO_REUSEPORT`** — universal untuk semua bahasa (Node, Go, Python, Rust, dll). Aplikasi anak harus dapat `listen(0.0.0.0:PORT)` dengan `SO_REUSEPORT` (mayoritas runtime modern sudah default). Daemon set env `RSPM_CLUSTER=1`, `RSPM_INSTANCE_ID=N`, `RSPM_INSTANCES=N`, `RSPM_EXEC_MODE=cluster_mode`.

Untuk Node app: set `process.env.NODE_APP_INSTANCE` tetap di-set untuk kompatibilitas.

### Soft Reload

PM2 soft reload kirim `'shutdown'` message ke worker via Node IPC, tunggu `disconnect` event, baru SIGINT. RSPM tidak punya channel IPC ke child (universal), jadi:

1. Spawn instance baru dengan instance_index sama.
2. Tunggu instance baru tulis `$RSPM_READY_FILE` (kalau `wait_ready: true`) atau sleep 100ms (default).
3. Stop instance lama (SIGINT + `kill_timeout_ms`).

Konsekuensi: app perlu opt-in `wait_ready: true` + tulis ready file untuk true zero-downtime.

### Log Format

PM2 punya banyak format (`raw`, `json`, format timestamp via `log_date_format`). RSPM v0.0.1:
- `prefix_timestamp: true` + `log_date_format`: prepend timestamp ke setiap baris.
- `merge_logs: true`: prepend `[<name>] ` ke setiap baris (instance suffix dihilangkan).
- Format JSON belum didukung (planned).

### Log Rotation

PM2 delegate ke modul `pm2-logrotate`. RSPM built-in: default 10 MB per file × 10 archives, otomatis rotate dengan rename `app-out.log` → `app-out.1.log` → `app-out.2.log` → dst.

### IPC Protocol

PM2 pakai `pm2-axon-rpc` (binary). RSPM pakai length-prefixed JSON frame (16 MiB max). Tidak ada interop binary-level; rewrite tooling pihak ketiga kalau langsung depend ke `pm2-axon`.

### Dump File

PM2: `~/.pm2/dump.pm2` (Node serialization).
RSPM: `$RSPM_HOME/dump.rspm` (JSON pretty-print).

Migrasi manual: dump PM2 ke JSON via `pm2 jlist > old.json`, edit field nama bila perlu, lalu sebagian besar field bisa langsung dipakai di `rspm` (extract `pm2_env` per proses).

## Step-by-Step Migrasi

1. Install `rspm` binary ke `$PATH`.
2. Stop daemon PM2: `pm2 kill`.
3. Set `RSPM_HOME` (opsional, default `~/.rspm`).
4. Salin `ecosystem.config.js` Anda apa adanya.
5. Start: `rspm start ecosystem.config.js`.
6. Verifikasi: `rspm list`, `rspm logs`, `rspm ping`.
7. Persist + bootable: `rspm save && sudo rspm startup --service rspm`.

## Yang BELUM Tersedia (Roadmap)

| Fitur | Target |
|-------|--------|
| TUI dashboard (`monit`) | v0.2.0 |
| Module install/uninstall | v0.3.0 |
| Deploy (SSH) | v0.3.0 |
| HTTP API (optional feature) | v0.4.0 |
| pm2.io / Keymetrics integration | tidak dalam scope |
| Native Windows service | post-1.0 |

## Bug Reports

Bug parity (RSPM behavior berbeda dari PM2 di area yang seharusnya identik): buka issue dengan reproducer minimal + output `pm2 jlist` dan `rspm jlist`.
