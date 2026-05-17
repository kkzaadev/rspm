# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

---

## [0.0.2] - 2026-05-17

### Added

- New parity commands: `describe` / `show` / `info`, `id`, `pid`, `env`,
  `flush`, `reset`, `reloadLogs`, `scale`.
- Daemon RPC variants: `Describe`, `Env`, `Scale`, `Flush`, `Reset`,
  `ReloadLogs`.
- `ProcessDetail` response payload mirroring PM2 `pm2 describe` output.
- npm wrapper package under `npm/` with `postinstall` downloader so
  `npm i -g rspm`, `bunx rspm`, and `pnpm add -g rspm` work on Linux.
- Linux aarch64 build target in the release workflow.
- Test mirrors for the new RPCs under `tests/tests/describe_env_id_pid.rs`
  and `tests/tests/scale.rs`.
- Canonical English `README.md` and Indonesian translation under
  `docs/i18n/id/README.md`.
- Root `CHANGELOG.md`.
- Manual benchmark scripts under `benchmarks/`.

### Changed

- Release workflow now produces platform-suffixed tarballs
  (`rspm-linux-x86_64.tar.gz`, `rspm-linux-aarch64.tar.gz`) so the npm
  postinstall script can pull binaries by predictable URL.
- README now documents feature status, installation paths, quick-start flow,
  app testing examples, state layout, and known PM2 parity gaps.

### Fixed

- Fixed benchmark wrapper functions so `PM2_BIN=pm2` calls the external PM2
  binary instead of recursively calling the shell function.
- CI `cargo-deny` step no longer passes the unsupported `--all-features`
  argument (broken since cargo-deny-action v0.18).
- `crates/rspm-core/src/types/app.rs` doc link now resolves
  (`crate::defaults::EXP_BACKOFF_CAP_MS`).

---

## [0.0.1] - 2026-05-17

### Added

- Initial Rust workspace for RSPM.
- CLI binary named `rspm`.
- Daemon auto-start over Unix domain sockets.
- Process lifecycle commands: `start`, `stop`, `restart`, `reload`, and
  `delete`.
- Process inspection commands: `list`, `jlist`, `prettylist`, and `logs`.
- Persistence commands: `save`, `dump`, and `resurrect`.
- Daemon control commands: `ping`, `kill`, and `send-signal`.
- Startup integration commands: `startup` and `unstartup`.
- Config loading for TOML, YAML, JSON, and basic `ecosystem.config.js`.
- PM2-like home layout under `$RSPM_HOME` or `~/.rspm`.
- Log capture for managed app stdout and stderr.
- Development docs for architecture, protocol, PM2 migration, and CLI behavior.

### Known Gaps

- PM2-style boxed and colored CLI output is not implemented yet.
- `prettylist` currently shares the same compact table renderer as `list`.
- `logs` does not yet provide full PM2-style live follow behavior.
- `monit`, `describe`, `flush`, module management, deploy, and dashboard are not
  implemented yet.
- Cluster mode is not full parity with PM2's Node cluster behavior yet.

---

[Unreleased]: https://github.com/kkzaadev/rspm/compare/v0.0.2...HEAD
[0.0.2]: https://github.com/kkzaadev/rspm/releases/tag/v0.0.2
[0.0.1]: https://github.com/kkzaadev/rspm/releases/tag/v0.0.1
