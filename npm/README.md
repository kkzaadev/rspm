# rspm

> Rust Process Manager — PM2 functional parity, written in Rust.

```bash
npm install -g rspm
# or
bun add -g rspm
# or
pnpm add -g rspm
```

After install, the `rspm` binary is on your `PATH`:

```bash
rspm start app.js --name api
rspm list
rspm restart api
rspm logs api --lines 200
rspm save        # persist current process list
rspm resurrect   # restart everything from the dump file
rspm describe 0  # PM2-style detailed view
rspm scale api 4 # cluster-mode resize
```

## How it works

This package is a thin Node.js wrapper. The first time you install it, a
`postinstall` script downloads the platform-specific binary from the matching
[GitHub Release](https://github.com/kkzaadev/rspm/releases) and unpacks it into
`./bin`. The `rspm` command on your `PATH` is a small Node shim that `execve`s
that native binary.

## Supported platforms

- Linux x86_64 ✅
- Linux aarch64 ✅
- macOS / Windows — roadmap (v0.2.0+)

If you need to build from source today, install via cargo:

```bash
cargo install --git https://github.com/kkzaadev/rspm
```

## Environment overrides

- `RSPM_SKIP_DOWNLOAD=1` — skip the postinstall download (useful in CI where
  you provide your own binary).
- `RSPM_VERSION=x.y.z` — override the binary version pulled by postinstall.

## License

MIT. See the [main repository](https://github.com/kkzaadev/rspm) for source and
issue tracking.
