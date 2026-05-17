# Fixtures — Mirror of `pm2/test/fixtures/`

Shell-script equivalents of the Node-only PM2 fixtures. Each script is small
enough to inline-read; keep behavior in sync with the PM2 counterpart so the
rspm parity test suite can assert against identical outcomes.

| File | PM2 counterpart | Behavior |
|------|------------------|----------|
| `echo.sh` | `pm2/test/fixtures/echo.js` | Print one line then sleep |
| `throw.sh` | `pm2/test/fixtures/auto-restart/throw.js` | Crash immediately (exit 1) |
| `throw-stable.sh` | `pm2/test/fixtures/exp-backoff/throw-stable.js` | Run a while, then crash |
| `delayed-exit.sh` | `pm2/test/fixtures/delayed_exit.js` | Exit 0 after a delay |
| `mem-hog.sh` | `pm2/test/fixtures/big-array.js` | Allocate then sleep |
| `env-print.sh` | `pm2/test/fixtures/env.js` | Print env vars and exit |
| `graceful.sh` | `pm2/test/fixtures/graceful.js` | Trap SIGINT, exit 0 |
| `ignore-sigint.sh` | – | Ignore SIGINT — force kill_timeout path |
| `sleeper.sh` | `pm2/test/fixtures/child.js` | Long-running silent process |
