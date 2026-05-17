#!/bin/sh
# E2E parity with pm2/test/e2e/cli/reload.sh.
# reload on a fork-mode app should fall back to restart and bump restart_time.
set -eu

RSPM_BIN="${RSPM_BIN:-./target/debug/rspm}"
HOME_DIR="$(mktemp -d)"
trap 'RSPM_HOME="$HOME_DIR" "$RSPM_BIN" kill || true; rm -rf "$HOME_DIR"' EXIT

export RSPM_HOME="$HOME_DIR"
export RSPM_DAEMON_BIN="$RSPM_BIN"

"$RSPM_BIN" start /bin/sh -- -c 'while true; do sleep 0.1; done'
BEFORE=$("$RSPM_BIN" jlist | python3 -c 'import json,sys; print(json.load(sys.stdin)[0]["restart_time"])')
"$RSPM_BIN" reload all
AFTER=$("$RSPM_BIN" jlist | python3 -c 'import json,sys; print(json.load(sys.stdin)[0]["restart_time"])')
test "$AFTER" -gt "$BEFORE"
"$RSPM_BIN" delete all
echo "OK: reload"
