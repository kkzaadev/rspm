#!/bin/sh
# E2E parity with pm2/test/e2e/cli/resurrect.sh.
set -eu

RSPM_BIN="${RSPM_BIN:-./target/debug/rspm}"
HOME_DIR="$(mktemp -d)"
trap 'RSPM_HOME="$HOME_DIR" "$RSPM_BIN" kill || true; rm -rf "$HOME_DIR"' EXIT

export RSPM_HOME="$HOME_DIR"
export RSPM_DAEMON_BIN="$RSPM_BIN"

"$RSPM_BIN" start /bin/sh -- -c 'while true; do sleep 0.1; done'
"$RSPM_BIN" save
"$RSPM_BIN" delete all
"$RSPM_BIN" resurrect
"$RSPM_BIN" jlist | grep -q '"online"'
"$RSPM_BIN" delete all
echo "OK: resurrect"
