const intervalMs = Number(process.env.BENCH_INTERVAL_MS || 1000);

setInterval(() => {
  if (process.env.BENCH_NOOP_LOG === "1") {
    return;
  }
}, intervalMs);
