# RSPM

Bahasa: [English](../../../README.md) | Bahasa Indonesia

RSPM adalah process manager berbasis Rust yang mengambil inspirasi dari PM2.
Project ini menjalankan daemon di background, mengelola aplikasi long-running,
menangkap log stdout/stderr, dan menyediakan CLI bernama `rspm`.

Status project saat ini masih awal (`0.0.1`). Beberapa command utama sudah bisa
dipakai, tapi tampilan CLI dan sebagian perilaku PM2 belum sepenuhnya parity.

Terjemahan disimpan di `docs/i18n/` supaya root repository tetap rapi.

## Status Fitur

Sudah tersedia:

- daemon auto-start lewat Unix domain socket
- `start`, `stop`, `restart`, `reload`, `delete`
- `list`, `jlist`, `prettylist`
- `logs`
- `save`, `dump`, `resurrect`
- `ping`, `kill`, `send-signal`
- `startup`, `unstartup`
- loader config TOML, YAML, JSON, dan basic `ecosystem.config.js`
- layout state mirip PM2 di `$RSPM_HOME` atau `~/.rspm`

Belum selesai:

- tampilan `list` belum seperti PM2; saat ini masih tabel plain
- `prettylist` masih memakai renderer yang sama dengan `list`
- `logs` belum live-follow penuh seperti PM2
- `monit`, `describe`, `flush`, module management, deploy, dan dashboard belum ada
- cluster mode belum full parity dengan Node cluster milik PM2
- target utama saat ini adalah Unix-like OS

Directory `pm2/` adalah referensi source PM2 yang dibaca untuk parity. RSPM
tidak menjalankan PM2 di runtime.

## Requirement

- Rust `1.95.0` atau lebih baru
- Unix-like OS untuk socket dan signal
- Node.js hanya diperlukan kalau menjalankan app `.js` atau contoh Node

Workspace ini memakai Rust edition 2024.

## Menjalankan Tanpa Install

Kalau binary `rspm` belum ada di shell, jalankan lewat Cargo:

```bash
cargo run -p rspm-cli --bin rspm -- --help
```

Format umumnya:

```bash
cargo run -p rspm-cli --bin rspm -- <command> [args...]
```

Contoh:

```bash
cargo run -p rspm-cli --bin rspm -- list
cargo run -p rspm-cli --bin rspm -- ping
```

Semua argumen setelah `--` dikirim ke binary `rspm`.

## Build Binary Lokal

Build CLI:

```bash
cargo build -p rspm-cli --bin rspm
```

Jalankan binary hasil build:

```bash
./target/debug/rspm --help
./target/debug/rspm list
```

Kalau menjalankan `rspm list` dan muncul:

```text
zsh: command not found: rspm
```

artinya binary belum terinstall di `PATH`. Gunakan `./target/debug/rspm`, atau
install dulu dengan Cargo.

## Install Command `rspm`

Install ke `$HOME/.cargo/bin`:

```bash
cargo install --path crates/rspm-cli --force
```

Pastikan Cargo bin masuk `PATH`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Untuk zsh agar permanen:

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

Cek:

```bash
which rspm
rspm --help
```

## Quick Start

Gunakan `RSPM_HOME` sementara saat testing supaya state asli di `~/.rspm` tidak
terganggu:

```bash
export RSPM_HOME=/tmp/rspm-demo
```

Start contoh app:

```bash
cargo run -p rspm-cli --bin rspm -- start ./examples/long-running.sh --name demo
```

Lihat proses:

```bash
cargo run -p rspm-cli --bin rspm -- list
```

Lihat log:

```bash
cargo run -p rspm-cli --bin rspm -- logs demo --lines 20
```

Restart, stop, dan delete:

```bash
cargo run -p rspm-cli --bin rspm -- restart demo
cargo run -p rspm-cli --bin rspm -- stop demo
cargo run -p rspm-cli --bin rspm -- delete demo
```

Matikan daemon:

```bash
cargo run -p rspm-cli --bin rspm -- kill
```

Kalau sudah build binary lokal, command yang sama bisa ditulis lebih pendek:

```bash
export RSPM_HOME=/tmp/rspm-demo
./target/debug/rspm start ./examples/long-running.sh --name demo
./target/debug/rspm list
./target/debug/rspm logs demo --lines 20
./target/debug/rspm kill
```

## Test Dengan Aplikasi Sendiri

Node.js:

```bash
export RSPM_HOME=/tmp/rspm-node-test
./target/debug/rspm start ./server.js --name api --cwd /path/ke/project -- --port 3000
./target/debug/rspm list
./target/debug/rspm logs api --lines 50
```

Shell script:

```bash
export RSPM_HOME=/tmp/rspm-shell-test
./target/debug/rspm start ./scripts/worker.sh --name worker
./target/debug/rspm logs worker --lines 50
```

Python:

```bash
export RSPM_HOME=/tmp/rspm-python-test
./target/debug/rspm start ./app.py --name python-app
./target/debug/rspm list
```

Auto-detect interpreter saat ini:

- `.js`, `.mjs`, `.cjs` dijalankan dengan `node`
- `.py` dijalankan dengan `python3`
- file lain dijalankan langsung sebagai executable

Kalau butuh interpreter manual:

```bash
./target/debug/rspm start ./app.custom --name custom --interpreter /usr/bin/env
```

## Ecosystem Config

TOML:

```bash
./target/debug/rspm start ./examples/apps.toml
```

PM2-style ecosystem file:

```bash
./target/debug/rspm start ./examples/ecosystem.config.js
```

Contoh minimal:

```js
module.exports = {
  apps: [
    {
      name: "api",
      script: "server.js",
      autorestart: true
    }
  ]
};
```

## Command Umum

```bash
rspm start <script-or-config> [--name NAME] [--cwd DIR] [--interpreter PATH] [-- ARGS...]
rspm stop <id|name|all>
rspm restart <id|name|all>
rspm reload <id|name|all>
rspm delete <id|name|all>
rspm list
rspm jlist
rspm prettylist
rspm logs [id|name] --lines 100
rspm save
rspm resurrect
rspm ping
rspm kill
rspm send-signal <SIGNAL> <id|name|all>
```

Gunakan help dari binary untuk memastikan command yang diterima build saat ini:

```bash
rspm --help
rspm start --help
```

## State Dan Log

RSPM menyimpan state di `$RSPM_HOME`. Kalau tidak diset, default-nya
`~/.rspm`.

Layout:

```text
$RSPM_HOME/
  logs/          stdout/stderr app
  pids/          pid file daemon dan app
  modules/       reserved untuk module support
  pm2.log        log daemon
  pm2.pid        pid daemon
  rpc.sock       socket request CLI
  pub.sock       socket event
  dump.rspm      process list yang disimpan
  dump.rspm.bak  backup dump sebelumnya
```

Untuk reset state test:

```bash
./target/debug/rspm kill
rm -rf /tmp/rspm-demo
```

## Development

Format:

```bash
cargo fmt --all
```

Check:

```bash
cargo check --workspace
```

Lint:

```bash
cargo clippy --all-targets -- -D warnings
```

Test:

```bash
cargo test --workspace
```

Catatan: beberapa test daemon, socket, dan process supervision lebih akurat
dijalankan dari terminal normal, bukan sandbox terbatas.

## Benchmark

RSPM punya script benchmark manual di `benchmarks/`.

Benchmark RSPM saja:

```bash
./benchmarks/benchmark-rspm.sh
```

Compare RSPM dengan PM2:

```bash
./benchmarks/compare-rspm-pm2.sh
```

Script compare membutuhkan PM2 yang sudah terinstall. Output CSV mentah ditulis
ke `target/benchmarks/`.

## Dokumen Terkait

- `CHANGELOG.md` - catatan perubahan project
- `benchmarks/README.md` - script benchmark dan workflow compare PM2
- `prd.md` - requirement dan rencana parity PM2
- `docs/architecture.md` - arsitektur crate dan daemon
- `docs/protocol.md` - format IPC
- `docs/migration-from-pm2.md` - catatan migrasi dari PM2
- `docs/cli-reference.md` - referensi CLI tambahan

## License

MIT
