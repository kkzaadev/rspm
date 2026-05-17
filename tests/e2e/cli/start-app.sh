#!/bin/sh
# E2E parity with pm2/test/e2e/cli/start-app.sh.
# Verifies the rspm binary auto-spawns the daemon on first contact and that
# `start` + `list` + `delete` round-trip works end-to-end.
set -eu

RSPM_BIN="${RSPM_BIN:-./target/debug/rspm}"
HOME_DIR="$(mktemp -d)"
trap 'RSPM_HOME="$HOME_DIR" "$RSPM_BIN" kill || true; rm -rf "$HOME_DIR"' EXIT

export RSPM_HOME="$HOME_DIR"
export RSPM_DAEMON_BIN="$RSPM_BIN"

"$RSPM_BIN" ping | grep -qi pong

"$RSPM_BIN" start /bin/sh -- -c 'while true; do sleep 0.1; done'
"$RSPM_BIN" list
"$RSPM_BIN" jlist | grep -q '"name"'
"$RSPM_BIN" stop all
"$RSPM_BIN" delete all
test "$("$RSPM_BIN" jlist | tr -d '[:space:]')" = "[]"
echo "OK: start-app"
