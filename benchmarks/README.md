# Benchmarks

This directory contains manual benchmark scripts for RSPM.

The benchmarks are intentionally not part of `cargo test`: they start daemons,
spawn real child processes, write state under temporary homes, and optionally
compare against PM2.

## Files

- `benchmark-rspm.sh` - benchmark RSPM only.
- `compare-rspm-pm2.sh` - compare RSPM with PM2 using the same fixture apps.
- `fixtures/bench-app.js` - long-running Node fixture.
- `fixtures/log-many.js` - log-heavy Node fixture for `logs --lines` timing.

## Requirements

- Rust toolchain for building `target/debug/rspm`.
- Node.js for benchmark fixtures.
- PM2 only for `compare-rspm-pm2.sh`.

Install PM2 if you want the comparison script:

```bash
npm install -g pm2
```

## RSPM-only benchmark

```bash
./benchmarks/benchmark-rspm.sh
```

Useful environment variables:

```bash
RUNS=10 APPS=25 ./benchmarks/benchmark-rspm.sh
RSPM_BIN=./target/release/rspm ./benchmarks/benchmark-rspm.sh
OUT_DIR=/tmp/rspm-bench ./benchmarks/benchmark-rspm.sh
```

## RSPM vs PM2 benchmark

```bash
./benchmarks/compare-rspm-pm2.sh
```

Useful environment variables:

```bash
RUNS=10 APPS=25 ./benchmarks/compare-rspm-pm2.sh
PM2_BIN=/path/to/pm2 ./benchmarks/compare-rspm-pm2.sh
RSPM_BIN=./target/release/rspm ./benchmarks/compare-rspm-pm2.sh
```

## What Is Measured

Both scripts measure wall-clock CLI latency in milliseconds for these cases:

- `cold_list` - first `list` call, including daemon auto-start.
- `warm_list` - repeated `list` calls against an already running daemon.
- `start_single` - starting one long-running app.
- `list_with_apps` - listing after multiple apps are running.
- `jlist_with_apps` - JSON process listing after multiple apps are running.
- `restart_single` - restarting one app repeatedly.
- `logs_tail` - reading the last 50 log lines.
- `delete_all` - deleting a batch of apps.

The scripts print a summary table and write raw CSV results to
`target/benchmarks/`.

## Notes

- Use the same machine, shell, build profile, and `RUNS`/`APPS` values when
  comparing results.
- Prefer `RSPM_BIN=./target/release/rspm` for release-like numbers.
- PM2 and RSPM use separate temporary homes, so the benchmark does not touch
  your normal `~/.pm2` or `~/.rspm` state.
- The scripts clean up daemons and temporary homes on exit.
