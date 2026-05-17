const total = Number(process.env.BENCH_LOG_LINES || 100);

for (let index = 0; index < total; index += 1) {
  console.log(`bench log line ${index}`);
}

setInterval(() => {}, 1000);
