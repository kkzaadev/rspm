#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RSPM_BIN="${RSPM_BIN:-$ROOT_DIR/target/debug/rspm}"
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

now_ns() {
  local value
  value="$(date +%s%N 2>/dev/null || true)"
  if [[ "$value" =~ ^[0-9]+$ ]]; then
    printf '%s\n' "$value"
  else
    python3 -c 'import time; print(time.perf_counter_ns())'
  fi
}

CSV="$OUT_DIR/rspm-$(date +%Y%m%d-%H%M%S).csv"
printf 'manager,case,iteration,duration_ms,status\n' > "$CSV"

RSPM_HOME_DIR="$(mktemp -d "${TMPDIR:-/tmp}/rspm-bench.XXXXXX")"

run_rspm() {
  RSPM_HOME="$RSPM_HOME_DIR" RSPM_DAEMON_BIN="$RSPM_BIN" command "$RSPM_BIN" "$@"
}

cleanup() {
  run_rspm kill >/dev/null 2>&1 || true
  rm -rf "$RSPM_HOME_DIR"
}
trap cleanup EXIT

measure() {
  local case_name="$1"
  local iteration="$2"
  shift 2

  local start_ns end_ns status duration_ms
  start_ns="$(now_ns)"
  set +e
  "$@" >/dev/null 2>&1
  status=$?
  set -e
  end_ns="$(now_ns)"
  duration_ms=$(( (end_ns - start_ns) / 1000000 ))

  printf 'rspm,%s,%s,%s,%s\n' "$case_name" "$iteration" "$duration_ms" "$status" >> "$CSV"

  if [[ "$status" -ne 0 ]]; then
    echo "error: benchmark case '$case_name' failed with status $status" >&2
    echo "results so far: $CSV" >&2
    exit "$status"
  fi
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

echo "Running RSPM benchmark"
echo "RSPM_BIN=$RSPM_BIN"
echo "RSPM_HOME=$RSPM_HOME_DIR"
echo "RUNS=$RUNS APPS=$APPS"

measure cold_list 1 run_rspm list

for iteration in $(seq 1 "$RUNS"); do
  measure warm_list "$iteration" run_rspm list
done

for iteration in $(seq 1 "$RUNS"); do
  name="rspm-start-$iteration"
  measure start_single "$iteration" run_rspm start "$FIXTURE_APP" --name "$name"
  run_rspm delete "$name" >/dev/null 2>&1
done

for index in $(seq 1 "$APPS"); do
  run_rspm start "$FIXTURE_APP" --name "rspm-list-$index" >/dev/null 2>&1
done

for iteration in $(seq 1 "$RUNS"); do
  measure list_with_apps "$iteration" run_rspm list
  measure jlist_with_apps "$iteration" run_rspm jlist
done

run_rspm delete all >/dev/null 2>&1
run_rspm start "$FIXTURE_APP" --name rspm-restart >/dev/null 2>&1

for iteration in $(seq 1 "$RUNS"); do
  measure restart_single "$iteration" run_rspm restart rspm-restart
done

run_rspm delete rspm-restart >/dev/null 2>&1
run_rspm start "$FIXTURE_LOG" --name rspm-logs >/dev/null 2>&1
sleep 1

for iteration in $(seq 1 "$RUNS"); do
  measure logs_tail "$iteration" run_rspm logs rspm-logs --lines 50
done

run_rspm delete rspm-logs >/dev/null 2>&1

for index in $(seq 1 "$APPS"); do
  run_rspm start "$FIXTURE_APP" --name "rspm-delete-$index" >/dev/null 2>&1
done
measure delete_all 1 run_rspm delete all

echo
summarize
echo
echo "CSV written to $CSV"
