# RSPM CLI Reference

Versi `0.0.1`. Binary: `rspm`. Daemon binary: `rspm-daemon` (sama executable, di-invoke via `rspm --daemon`).

## Global Behavior

- Saat command pertama dijalankan, CLI auto-spawn daemon kalau `$RSPM_HOME/rpc.sock` tidak ada / tidak responsif.
- `$RSPM_HOME` mengontrol lokasi state (default `~/.rspm`).
- `$RSPM_DAEMON_BIN` overrides daemon executable path (otomatis pakai `current_exe` kalau tidak diset).
- `$RUST_LOG=rspm=debug` aktifkan trace logging di daemon log file (`$RSPM_HOME/pm2.log`).

## Subcommands

### Process Lifecycle

| Command | Alias | Tujuan |
|---------|-------|--------|
| `rspm start <script-or-config> [options]` | – | Start script (`.js` `.py` `.sh`) atau config file (`.toml` `.yaml` `.json` `ecosystem.config.js`) |
| `rspm stop <id\|name\|all>` | – | Stop matching process (SIGINT → `kill_timeout_ms` → SIGKILL) |
| `rspm restart <id\|name\|all>` | – | Stop + spawn ulang, increment `restart_time` |
| `rspm reload <id\|name\|all>` | – | Cluster: rolling reload zero-downtime. Fork: fallback ke restart. |
| `rspm delete <id\|name\|all>` | `del` | Stop + remove dari registry |

#### `start` Flags

```
-n, --name <NAME>            Override nama (default = stem dari script)
    --cwd <DIR>              Working directory
    --interpreter <PATH>     Path interpreter (override auto-detect)
-i, --instances <N|max|-1>   Jumlah instance (cluster mode)
    --no-autorestart         Disable auto restart pada exit non-zero
-- <ARGS...>                 Argumen yang diteruskan ke script (setelah --)
```

Contoh:
```bash
rspm start server.js --name api -i 4 --no-autorestart -- --port 3000
rspm start ecosystem.config.js
rspm start apps.toml
```

### Inspection

| Command | Alias | Tujuan |
|---------|-------|--------|
| `rspm list` | `ls` (default) | Tabel ringkas semua proses (id, name, status, uptime, restart_time, CPU, MEM) |
| `rspm jlist` | – | Output JSON array (pipe-friendly) |
| `rspm prettylist` | – | Tabel lebih lengkap (script path, env count, dll) |
| `rspm logs [id\|name] [--lines N]` | `log` | Tail terakhir N baris (default 100). Saat ini bukan real-time follow. |

### Persistence

| Command | Tujuan |
|---------|--------|
| `rspm save` | Tulis daftar app saat ini ke `$RSPM_HOME/dump.rspm` |
| `rspm dump` | Alias untuk `save` |
| `rspm resurrect` | Baca `dump.rspm` → start setiap app (skip kalau sudah jalan) |

### Daemon Control

| Command | Tujuan |
|---------|--------|
| `rspm ping` | Health check; balas "pong" |
| `rspm kill` | Stop semua child + shutdown daemon (rpc.sock dihapus) |
| `rspm send-signal <SIGNAL> <id\|name\|all>` | Kirim sinyal Unix (`SIGTERM`, `SIGUSR1`, dll) ke matching pid |

### System Integration

| Command | Tujuan |
|---------|--------|
| `rspm startup [--platform <SYS>] [--user <USER>] [--service <NAME>]` | Generate + install init unit (systemd / openrc / sysv). Detect otomatis kalau `--platform` kosong. Butuh sudo untuk write file. |
| `rspm unstartup [...same flags]` | Disable + remove unit file |

`--platform` accept: `systemd`, `openrc`, `sysv`. `--service` default `rspm`. `--user` default = user yang menjalankan command.

## Exit Codes

| Code | Arti |
|------|------|
| `0` | Success |
| `1` | Generic error (lihat stderr) |
| `2` | Argument parsing error (clap) |

CLI sengaja tidak memetakan setiap kategori error ke exit code unik di v0.0.1. Detail lengkap ada di stderr.

## Environment Variables

| Variable | Default | Efek |
|----------|---------|------|
| `RSPM_HOME` | `~/.rspm` | Override directory state |
| `RSPM_DAEMON_BIN` | `current_exe()` | Override path binary daemon |
| `RUST_LOG` | – | Tracing level (`rspm=debug`, `info`, dll) |

## Output Format

`list` / `prettylist`: tabel ANSI (kalau TTY) dengan kolom `pm_id | name | status | uptime | restarts | CPU% | MEM`.

`jlist`: array JSON `ProcessInfo`, satu element per proses. Cocok untuk `jq`:
```bash
rspm jlist | jq '.[] | select(.status == "online") | .name'
```

`logs`: prefix `[name] [out|err] <line>`. Kalau tidak ada selector, tampilkan semua proses gabung.

## Examples

```bash
# Mulai cluster Node app dengan 4 instance + watch
rspm start server.js -n api -i 4

# Pakai ecosystem config (PM2-compatible)
rspm start ecosystem.config.js
rspm list

# Save state, simulate reboot, restore
rspm save
rspm kill
rspm start ./apps.toml   # auto-spawn daemon
rspm resurrect

# Production: install systemd unit
sudo rspm startup --service rspm

# Send SIGUSR1 (mis. untuk log rotate aplikasi sendiri)
rspm send-signal SIGUSR1 api
```

## Catatan Migrasi PM2

Mayoritas command sama. Perbedaan v0.0.1:
- `rspm monit` (TUI dashboard) belum ada.
- `rspm logs --follow` belum stream real-time di CLI (subscriber API ada via `EventSubscriber`).
- `rspm describe <id>` belum dibedakan dari `prettylist`.
- `rspm deploy` belum ada (Phase 16).
- `rspm install <module>` belum ada (Phase 15).

Lihat `migration-from-pm2.md` untuk tabel detail.
