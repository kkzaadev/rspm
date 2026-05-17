# CLAUDE.md — RSPM (Rust PM2)

> File ini dibaca otomatis tiap sesi. Selalu ikuti sebelum menulis kode.

## Tujuan

Membangun ulang PM2 (Process Manager 2) dalam Rust dengan **functional parity penuh**.

- Nama project: `rspm` (Rust Process Manager)
- Versi awal: `0.0.1` → target rilis terdekat `0.1.0` (staging/internal production)
- Lisensi: MIT
- Platform v0.0.x: **Linux only** (macOS/Windows menyusul, lihat PRD D5)
- Blueprint utama: `prd.md` di root project (~120 task atomik, T-prefix)

## Referensi Source Mutlak

Sebelum menulis kode untuk task apa pun, **WAJIB baca file PM2 referensi** yang tercantum di `prd.md` Section 8 (task DoD) atau Section 10 (critical files cheatsheet).

Folder `pm2/` adalah **READ-ONLY** salinan PM2 v5.x original. **Jangan modifikasi** apapun di dalamnya. Index referensi cepat ada di `prd.md` Section 11.

## Bahasa & Tooling

- Rust edition **2024** (pin via `rust-toolchain.toml`, toolchain `1.95.0`)
- Async runtime: `tokio` (multi-thread)
- CLI parsing: `clap` v4 (derive macros)
- Serialisasi: `serde` + `serde_json` + `serde_yaml` + `toml`
- Embedded JS (ecosystem.config.js): `boa_engine`
- File watcher: `notify`
- System info: `sysinfo`
- Unix syscall: `nix`
- Error: `thiserror` (library), `anyhow` (binary)
- Logging: `tracing` + `tracing-subscriber`
- TUI: `ratatui` + `crossterm`
- HTTP (optional feature): `axum`
- SSH: `russh`
- Templating: `tera`

## Workspace

Multi-crate cargo workspace. Lihat `prd.md` Section 4 untuk dependency graph lengkap.

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

**Aturan keras**: tidak boleh ada cyclic dependency. `rspm-core` tidak boleh import crate lain di workspace.

## Aturan Vibe Coding (Hard Rules)

R1. Selalu **TDD**: tulis test dulu (`#[cfg(test)] mod tests` atau `tests/*.rs`), baru implementasi.
R2. Jangan pernah `unwrap()` di kode produksi. Pakai `?` + `thiserror`. Untuk `Option::None` yang impossible, gunakan `.expect("invariant: <reason>")`.
R3. Jangan tambah dependency baru tanpa update `deny.toml`. Justifikasi di commit body.
R4. Setiap fungsi `pub` punya doc comment `///` + minimal 1 contoh kode jalan.
R5. Setiap PR/commit hanya menyelesaikan **1 task atomik** (T-prefix). Branch name: `task/T<phase>.<idx>-<kebab-summary>`.
R6. Field name di protocol/config **persis sama** dengan PM2 (snake_case), kecuali dicatat di `docs/migration-from-pm2.md`. Canonical: `pm2/lib/API/schema.json`.
R7. Kalau ragu antara "ikuti PM2" vs "lebih bersih ala Rust" → **selalu ikuti PM2** untuk v0.0.x. Catat divergence di migration doc.
R8. Tidak ada `panic!`, `process::exit`, atau `unwrap` di library crate. Hanya entry point (`rspm-cli`, `rspm-daemon` main) yang boleh.
R9. Semua path file pakai `PathBuf` (jangan `String`). Semua waktu pakai `chrono::DateTime<Utc>`. ProcessId: `u32`.
R10. Default value didefinisikan di `rspm_core::defaults` atau `rspm_core::constants`. **Tidak boleh hardcode magic number** di tempat lain.

## Direktori PM2 Home (Compat)

```
$RSPM_HOME (default ~/.rspm/)
├── pids/                 # per-app pid + ready sentinel
├── logs/                 # stdout/stderr per app
├── modules/              # installed modules
├── pm2.log               # daemon log (nama dipertahankan untuk kompat tooling)
├── pm2.pid               # daemon pid lock
├── rpc.sock              # IPC request/reply (Unix Domain Socket)
├── pub.sock              # IPC pub/sub events
├── dump.rspm             # persisted process list (JSON)
└── dump.rspm.bak         # backup sebelum save
```

## Konvensi Naming Internal

- `App` = definisi aplikasi (dari config user)
- `Process` = instance running (1 app cluster mode = N process)
- `ProcessId` (`pm_id` di PM2) = monotonic counter dari daemon, type `u32`
- `Bus` = pub/sub event channel (broadcast)
- `God` = daemon supervisor (nama dipertahankan dari PM2)
- `ManagedProcess` = entry di registry God (child + ProcessInfo + watcher handle)

## Workflow per Task

1. Baca task DoD dari `prd.md` (T-prefix).
2. Baca file PM2 referensi yang disebut task atau di Section 10.
3. **Tulis test dulu** di crate yang sama (unit) atau `tests/` (integration).
4. Implementasi minimal sampai test hijau.
5. Jalankan checklist berikut sebelum commit:
   ```
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --workspace
   cargo deny check          # jika tambah dep
   cargo doc --no-deps       # jika tambah pub API
   ```
6. Commit format: `feat(<crate>): T<phase>.<idx> <ringkasan>`. Body jelaskan WHY dan referensi PM2 yang dipakai.

## Threading Model

- Daemon: 1 tokio runtime multi-thread, default worker_threads = num_cpus.
- 1 task per: accept loop (rpc & pub), wait child, stream stdout/stderr, worker tick.
- State `God` dilindungi `tokio::sync::Mutex` (async-aware, bukan `std::sync::Mutex`).
- **Hindari hold mutex sambil await I/O panjang** (mis. tail file, network) — pegang sebentar, lepas, baru I/O.

## Error UX (CLI)

- Error ke user CLI **wajib actionable**: "process 'api' not found, coba `rspm list`" lebih baik daripada "process not found".
- Format: `error: <kategori>: <detail>\nhelp: <saran>`.
- Internal logging pakai `tracing`. Level INFO default, `RUST_LOG=rspm=debug` untuk debug.

## Stabilitas Wire Protocol

- `PROTOCOL_VERSION = 1` untuk seluruh v0.x.
- Client/daemon mismatch → reject di handshake dengan pesan jelas suggest `rspm update`.
- Breaking change → bump `PROTOCOL_VERSION` + catat di CHANGELOG.

## Saat Behavior PM2 Ambigu

1. Tulis test case yang menggambarkan behavior expected.
2. Cek source PM2 (`pm2/lib/...`) — replikasi 1:1.
3. Kalau test PM2 ada (`pm2/test/`), jalankan untuk konfirmasi.
4. Divergence (kalau terpaksa) → catat di `docs/migration-from-pm2.md`.

## State Implementasi (per 2026-05-17)

- ✅ Phase 0–11, 13 implemented (core, protocol, ipc, config, daemon, client, cli, worker, monitor, watcher, logs, startup).
- 🟡 Phase 12 partial — cluster mode pakai SO_REUSEPORT, soft reload zero-downtime sudah jalan, beberapa edge case masih dirapikan.
- ❌ Phase 14 (TUI), 15 (modules), 16 (deploy SSH), 17 (HTTP API) belum dikerjakan — di-defer ke v0.2.0+.
- Status snapshot lengkap: jalankan `cargo test --workspace` + cek `prd.md` Section 8.
