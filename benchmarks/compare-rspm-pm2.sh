#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RSPM_BIN="${RSPM_BIN:-$ROOT_DIR/target/debug/rspm}"
PM2_BIN="${PM2_BIN:-pm2}"
RUNS="${RUNS:-5}"
APPS="${APPS:-10}"
OUT_DIR="${OUT_DIR:-$ROOT_DIR/target/benchmarks}"
FIXTURE_APP="$ROOT_DIR/benchmarks/fixtures/bench-app.js"
FIXTURE_LOG="$ROOT_DIR/benchmarks/fixtures/log-many.js"

mkdir -p "$OUT_DIR"

if [[ ! -x "$RSPM_BIN" ]]; then
  cargo build -p rspm-cli --bin rspm
fi

if ! command -v node >/dev/null 2>&1; then
  echo "error: node is required for benchmark fixtures" >&2
  exit 1
fi

if ! command -v "$PM2_BIN" >/dev/null 2>&1; then
  echo "error: pm2 was not found" >&2
  echo "install it first, or pass PM2_BIN=/path/to/pm2" >&2
  exit 1
fi

now_ns() {
  local value
  value="$(date +%s%N 2>/dev/null || true)"
  if [[ "$value" =~ ^[0-9]+$ ]]; then
    printf '%s\n' "$value"
  else
    python3 -c 'import time; print(time.perf_counter_ns())'
  fi
}

CSV="$OUT_DIR/rspm-vs-pm2-$(date +%Y%m%d-%H%M%S).csv"
printf 'manager,case,iteration,duration_ms,status\n' > "$CSV"

RSPM_HOME_DIR="$(mktemp -d "${TMPDIR:-/tmp}/rspm-compare.XXXXXX")"
PM2_HOME_DIR="$(mktemp -d "${TMPDIR:-/tmp}/pm2-compare.XXXXXX")"

run_rspm() {
  RSPM_HOME="$RSPM_HOME_DIR" RSPM_DAEMON_BIN="$RSPM_BIN" command "$RSPM_BIN" "$@"
}

run_pm2() {
  PM2_HOME="$PM2_HOME_DIR" \
  PM2_DISABLE_VERSION_CHECK=1 \
  PM2_HIDE_USAGE=1 \
  command "$PM2_BIN" "$@"
}

cleanup() {
  run_rspm kill >/dev/null 2>&1 || true
  run_pm2 kill >/dev/null 2>&1 || true
  rm -rf "$RSPM_HOME_DIR" "$PM2_HOME_DIR"
}
trap cleanup EXIT

measure() {
  local manager="$1"
  local case_name="$2"
  local iteration="$3"
  shift 3

  local start_ns end_ns status duration_ms
  start_ns="$(now_ns)"
  set +e
  "$@" >/dev/null 2>&1
  status=$?
  set -e
  end_ns="$(now_ns)"
  duration_ms=$(( (end_ns - start_ns) / 1000000 ))

  printf '%s,%s,%s,%s,%s\n' "$manager" "$case_name" "$iteration" "$duration_ms" "$status" >> "$CSV"

  if [[ "$status" -ne 0 ]]; then
    echo "error: $manager benchmark case '$case_name' failed with status $status" >&2
    echo "results so far: $CSV" >&2
    exit "$status"
  fi
}

measure_logs() {
  local manager="$1"
  local iteration="$2"

  if [[ "$manager" == "rspm" ]]; then
    measure "$manager" logs_tail "$iteration" run_rspm logs rspm-logs --lines 50
  else
    measure "$manager" logs_tail "$iteration" run_pm2 logs pm2-logs --lines 50 --nostream
  fi
}

bench_rspm() {
  measure rspm cold_list 1 run_rspm list

  for iteration in $(seq 1 "$RUNS"); do
    measure rspm warm_list "$iteration" run_rspm list
  done

  for iteration in $(seq 1 "$RUNS"); do
    local name="rspm-start-$iteration"
    measure rspm start_single "$iteration" run_rspm start "$FIXTURE_APP" --name "$name"
    run_rspm delete "$name" >/dev/null 2>&1
  done

  for index in $(seq 1 "$APPS"); do
    run_rspm start "$FIXTURE_APP" --name "rspm-list-$index" >/dev/null 2>&1
  done

  for iteration in $(seq 1 "$RUNS"); do
    measure rspm list_with_apps "$iteration" run_rspm list
    measure rspm jlist_with_apps "$iteration" run_rspm jlist
  done

  run_rspm delete all >/dev/null 2>&1
  run_rspm start "$FIXTURE_APP" --name rspm-restart >/dev/null 2>&1

  for iteration in $(seq 1 "$RUNS"); do
    measure rspm restart_single "$iteration" run_rspm restart rspm-restart
  done

  run_rspm delete rspm-restart >/dev/null 2>&1
  run_rspm start "$FIXTURE_LOG" --name rspm-logs >/dev/null 2>&1
  sleep 1

  for iteration in $(seq 1 "$RUNS"); do
    measure_logs rspm "$iteration"
  done

  run_rspm delete rspm-logs >/dev/null 2>&1

  for index in $(seq 1 "$APPS"); do
    run_rspm start "$FIXTURE_APP" --name "rspm-delete-$index" >/dev/null 2>&1
  done
  measure rspm delete_all 1 run_rspm delete all
}

bench_pm2() {
  measure pm2 cold_list 1 run_pm2 list

  for iteration in $(seq 1 "$RUNS"); do
    measure pm2 warm_list "$iteration" run_pm2 list
  done

  for iteration in $(seq 1 "$RUNS"); do
    local name="pm2-start-$iteration"
    measure pm2 start_single "$iteration" run_pm2 start "$FIXTURE_APP" --name "$name"
    run_pm2 delete "$name" >/dev/null 2>&1
  done

  for index in $(seq 1 "$APPS"); do
    run_pm2 start "$FIXTURE_APP" --name "pm2-list-$index" >/dev/null 2>&1
  done

  for iteration in $(seq 1 "$RUNS"); do
    measure pm2 list_with_apps "$iteration" run_pm2 list
    measure pm2 jlist_with_apps "$iteration" run_pm2 jlist
  done

  run_pm2 delete all >/dev/null 2>&1
  run_pm2 start "$FIXTURE_APP" --name pm2-restart >/dev/null 2>&1

  for iteration in $(seq 1 "$RUNS"); do
    measure pm2 restart_single "$iteration" run_pm2 restart pm2-restart
  done

  run_pm2 delete pm2-restart >/dev/null 2>&1
  run_pm2 start "$FIXTURE_LOG" --name pm2-logs >/dev/null 2>&1
  sleep 1

  for iteration in $(seq 1 "$RUNS"); do
    measure_logs pm2 "$iteration"
  done

  run_pm2 delete pm2-logs >/dev/null 2>&1

  for index in $(seq 1 "$APPS"); do
    run_pm2 start "$FIXTURE_APP" --name "pm2-delete-$index" >/dev/null 2>&1
  done
  measure pm2 delete_all 1 run_pm2 delete all
}

summarize() {
  awk -F, '
    NR > 1 {
      key = $1 "," $2
      count[key] += 1
      sum[key] += $4
      if (!(key in min) || $4 < min[key]) min[key] = $4
      if (!(key in max) || $4 > max[key]) max[key] = $4
    }
    END {
      printf "%-8s %-24s %6s %10s %10s %10s\n", "manager", "case", "runs", "min_ms", "avg_ms", "max_ms"
      for (key in count) {
        split(key, parts, ",")
        printf "%-8s %-24s %6d %10.0f %10.2f %10.0f\n", parts[1], parts[2], count[key], min[key], sum[key] / count[key], max[key]
      }
    }
  ' "$CSV"
}

echo "Running RSPM vs PM2 benchmark"
echo "RSPM_BIN=$RSPM_BIN"
echo "PM2_BIN=$PM2_BIN"
echo "RSPM_HOME=$RSPM_HOME_DIR"
echo "PM2_HOME=$PM2_HOME_DIR"
echo "RUNS=$RUNS APPS=$APPS"

bench_rspm
bench_pm2

echo
summarize
echo
echo "CSV written to $CSV"
