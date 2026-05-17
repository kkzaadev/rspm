# RSPM

`rspm` is a Rust process manager built from the PM2 behavior blueprint in
`prd.md`. The current workspace implements the v0.0.1 core slice:

- daemon bootstrap over Unix domain sockets
- `start`, `stop`, `restart`, `delete`, `list`, `logs`, `save`, `ping`, and `kill`
- TOML, YAML, JSON, and basic `ecosystem.config.js` loading
- PM2-style home layout under `$RSPM_HOME` or `~/.rspm`

The `pm2/` directory is a read-only reference source tree.

## Usage

```bash
cargo run -p rspm-cli --bin rspm -- start ./examples/long-running.sh --name demo
cargo run -p rspm-cli --bin rspm -- list
cargo run -p rspm-cli --bin rspm -- logs demo --lines 50
cargo run -p rspm-cli --bin rspm -- stop demo
```

For development:

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

