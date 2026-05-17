#!/usr/bin/env node
// Wrapper that execs the native rspm binary downloaded by scripts/install.js.
// Keeps argv1 + signals + exit code straight through so the npm-installed CLI
// is indistinguishable from a hand-built `cargo install`.

import path from "node:path";
import fs from "node:fs";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const bin = path.join(
  __dirname,
  process.platform === "win32" ? "rspm.exe" : "rspm",
);
if (!fs.existsSync(bin)) {
  console.error(
    "rspm: native binary not found — run `npm rebuild rspm` to fetch it, " +
      "or install from source with `cargo install --git https://github.com/kkzaadev/rspm`.",
  );
  process.exit(127);
}

const result = spawnSync(bin, process.argv.slice(2), {
  stdio: "inherit",
});

if (result.signal) {
  process.kill(process.pid, result.signal);
} else {
  process.exit(result.status ?? 0);
}
