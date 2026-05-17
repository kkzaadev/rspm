#!/usr/bin/env node
// Downloads the platform-specific rspm binary from the matching GitHub Release
// and unpacks it into ./bin so the wrapper at bin/rspm.js can execute it.
//
// Skipped automatically when:
//   - RSPM_SKIP_DOWNLOAD=1 is set (CI / monorepo dev installs)
//   - the binary already exists (re-install)

"use strict";

const fs = require("node:fs");
const path = require("node:path");
const https = require("node:https");
const { execFileSync } = require("node:child_process");
const { pipeline } = require("node:stream/promises");

const pkg = require("../package.json");
const VERSION = process.env.RSPM_VERSION || pkg.version;
const REPO = "kkzaadev/rspm";
const BIN_DIR = path.join(__dirname, "..", "bin");
const BIN_NAMES = ["rspm", "rspm-daemon"];

function resolveTarget() {
  const platform = process.platform;
  const arch = process.arch;
  if (platform !== "linux") {
    throw new Error(
      `rspm v${VERSION} only ships Linux binaries (got ${platform}). ` +
        "macOS / Windows support is on the roadmap — install from source with cargo for now.",
    );
  }
  if (arch !== "x64") {
    throw new Error(
      `unsupported cpu arch: ${arch} (rspm v${VERSION} ships linux-x86_64 only; ` +
        "linux-aarch64 lands in v0.0.3)",
    );
  }
  return { platform, arch };
}

function downloadUrl(version, platform, arch) {
  const archLabel = arch === "x64" ? "x86_64" : "aarch64";
  const asset = `rspm-${platform}-${archLabel}.tar.gz`;
  return `https://github.com/${REPO}/releases/download/v${version}/${asset}`;
}

function get(url) {
  return new Promise((resolve, reject) => {
    https
      .get(url, { headers: { "User-Agent": `rspm-installer/${VERSION}` } }, (res) => {
        if (res.statusCode === 301 || res.statusCode === 302) {
          resolve(get(res.headers.location));
          return;
        }
        if (res.statusCode !== 200) {
          reject(
            new Error(
              `download failed: ${res.statusCode} ${res.statusMessage} for ${url}`,
            ),
          );
          return;
        }
        resolve(res);
      })
      .on("error", reject);
  });
}

async function main() {
  if (process.env.RSPM_SKIP_DOWNLOAD === "1") {
    console.log("rspm: RSPM_SKIP_DOWNLOAD=1, skipping binary download.");
    return;
  }

  if (BIN_NAMES.every((name) => fs.existsSync(path.join(BIN_DIR, name)))) {
    console.log("rspm: binaries already present, skipping download.");
    return;
  }

  fs.mkdirSync(BIN_DIR, { recursive: true });

  const { platform, arch } = resolveTarget();
  const url = downloadUrl(VERSION, platform, arch);
  console.log(`rspm: downloading ${url}`);

  const tarball = path.join(BIN_DIR, "rspm.tar.gz");
  const res = await get(url);
  await pipeline(res, fs.createWriteStream(tarball));

  // Extract via system `tar` to keep zero npm dependencies.
  execFileSync("tar", ["-xzf", tarball, "-C", BIN_DIR], { stdio: "inherit" });
  fs.unlinkSync(tarball);

  for (const name of BIN_NAMES) {
    const p = path.join(BIN_DIR, name);
    if (!fs.existsSync(p)) {
      throw new Error(`expected ${p} after extraction, not found`);
    }
    fs.chmodSync(p, 0o755);
  }
  console.log("rspm: install complete.");
}

main().catch((err) => {
  console.error(`rspm: install failed: ${err.message}`);
  process.exit(1);
});
