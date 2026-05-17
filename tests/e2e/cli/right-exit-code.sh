#!/bin/sh
# E2E parity: rspm CLI must return non-zero exit code on RPC error.
set -eu

RSPM_BIN="${RSPM_BIN:-./target/debug/rspm}"
HOME_DIR="$(mktemp -d)"
trap 'RSPM_HOME="$HOME_DIR" "$RSPM_BIN" kill || true; rm -rf "$HOME_DIR"' EXIT

export RSPM_HOME="$HOME_DIR"
export RSPM_DAEMON_BIN="$RSPM_BIN"

"$RSPM_BIN" ping
# Stop with selector that doesn't exist should fail.
if "$RSPM_BIN" stop 9999 2>/dev/null; then
  echo "FAIL: expected stop on missing id to error"
  exit 1
fi
echo "OK: right-exit-code"
