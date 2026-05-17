# RSPM

Language: English | [Bahasa Indonesia](docs/i18n/id/README.md)

RSPM is a Rust process manager inspired by PM2. It runs a background daemon,
supervises long-running applications, captures stdout/stderr logs, and exposes a
CLI named `rspm`.

The project is still early (`0.0.1`). The core daemon and common PM2-style
commands are available, but the CLI presentation and some PM2 parity behavior
are not complete yet.

Translations are kept under `docs/i18n/` so the repository root stays clean.

## Feature Status

Available today:

- daemon auto-start over Unix domain sockets
- `start`, `stop`, `restart`, `reload`, `delete`
- `list`, `jlist`, `prettylist`
- `logs`
- `save`, `dump`, `resurrect`
- `ping`, `kill`, `send-signal`
- `startup`, `unstartup`
- TOML, YAML, JSON, and basic `ecosystem.config.js` loading
- PM2-like state layout under `$RSPM_HOME` or `~/.rspm`

Not finished yet:

- `list` still renders a plain compact table, not PM2's boxed/colored UI
- `prettylist` currently uses the same renderer as `list`
- `logs` does not yet provide full PM2-style live follow behavior
- `monit`, `describe`, `flush`, module management, deploy, and dashboard are not implemented
- cluster mode is not full parity with PM2's Node cluster behavior yet
- the current target is Unix-like systems

The `pm2/` directory is a read-only PM2 source reference used for parity work.
RSPM does not run PM2 at runtime.

## Requirements

- Rust `1.95.0` or newer
- Unix-like OS for sockets and signals
- Node.js only if you want to run `.js` apps or the Node examples

This workspace uses Rust edition 2024.

## Run Without Installing

If the `rspm` command is not available in your shell yet, run it through Cargo:

```bash
cargo run -p rspm-cli --bin rspm -- --help
```

General form:

```bash
cargo run -p rspm-cli --bin rspm -- <command> [args...]
```

Examples:

```bash
cargo run -p rspm-cli --bin rspm -- list
cargo run -p rspm-cli --bin rspm -- ping
```

Everything after `--` is passed to the `rspm` binary.

## Build A Local Binary

Build the CLI:

```bash
cargo build -p rspm-cli --bin rspm
```

Run the built binary:

```bash
./target/debug/rspm --help
./target/debug/rspm list
```

If this fails:

```text
zsh: command not found: rspm
```

it means `rspm` is not installed in your `PATH`. Use `./target/debug/rspm`, or
install the CLI with Cargo.

## Install The `rspm` Command

Install into Cargo's bin directory:

```bash
cargo install --path crates/rspm-cli --force
```

Make sure Cargo's bin directory is in `PATH`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

For zsh, make it permanent:

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

Verify:

```bash
which rspm
rspm --help
```

## Quick Start

Use a temporary `RSPM_HOME` while testing so your real `~/.rspm` state is not
touched:

```bash
export RSPM_HOME=/tmp/rspm-demo
```

Start the example app:

```bash
cargo run -p rspm-cli --bin rspm -- start ./examples/long-running.sh --name demo
```

List processes:

```bash
cargo run -p rspm-cli --bin rspm -- list
```

Read logs:

```bash
cargo run -p rspm-cli --bin rspm -- logs demo --lines 20
```

Restart, stop, and delete:

```bash
cargo run -p rspm-cli --bin rspm -- restart demo
cargo run -p rspm-cli --bin rspm -- stop demo
cargo run -p rspm-cli --bin rspm -- delete demo
```

Shutdown the daemon:

```bash
cargo run -p rspm-cli --bin rspm -- kill
```

The same flow with a local binary:

```bash
export RSPM_HOME=/tmp/rspm-demo
./target/debug/rspm start ./examples/long-running.sh --name demo
./target/debug/rspm list
./target/debug/rspm logs demo --lines 20
./target/debug/rspm kill
```

## Test With Your Own App

Node.js:

```bash
export RSPM_HOME=/tmp/rspm-node-test
./target/debug/rspm start ./server.js --name api --cwd /path/to/project -- --port 3000
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

Current interpreter detection:

- `.js`, `.mjs`, and `.cjs` run with `node`
- `.py` runs with `python3`
- other files are executed directly

Provide an interpreter manually when needed:

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

Minimal `ecosystem.config.js`:

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

## Common Commands

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

Use the binary help to verify the commands accepted by the current build:

```bash
rspm --help
rspm start --help
```

## State And Logs

RSPM stores state in `$RSPM_HOME`. If it is not set, RSPM uses `~/.rspm`.

Layout:

```text
$RSPM_HOME/
  logs/          app stdout/stderr logs
  pids/          daemon and app pid files
  modules/       reserved for module support
  pm2.log        daemon log
  pm2.pid        daemon pid file
  rpc.sock       CLI request socket
  pub.sock       event socket
  dump.rspm      saved process list
  dump.rspm.bak  previous dump backup
```

Reset test state:

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

Some daemon, socket, and process supervision behavior is more accurate from a
normal terminal than from a restricted sandbox.

## Benchmarks

RSPM includes manual benchmark scripts under `benchmarks/`.

Run the RSPM-only benchmark:

```bash
./benchmarks/benchmark-rspm.sh
```

Compare RSPM with PM2:

```bash
./benchmarks/compare-rspm-pm2.sh
```

The comparison script requires PM2 to be installed separately. Raw CSV output is
written to `target/benchmarks/`.

## Related Docs

- `CHANGELOG.md` - notable project changes
- `benchmarks/README.md` - benchmark scripts and PM2 comparison workflow
- `prd.md` - requirements and PM2 parity plan
- `docs/architecture.md` - crate and daemon architecture
- `docs/protocol.md` - IPC format
- `docs/migration-from-pm2.md` - PM2 migration notes
- `docs/cli-reference.md` - additional CLI reference

## License

MIT
