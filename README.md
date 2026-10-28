# Whereabouts

Let your AI agent ask **your phone** where you are, on demand. You host the relay. Sign-in and user data live in Supabase.

```
agent ──HTTP / MCP──▶ relay (Rust) ──WebSocket──▶ phone app (Tauri) ──GPS fix──▶ relay ──▶ agent
```

The relay keeps no location history. It sends a ping, the phone reads its GPS now, and the answer goes back to the agent.

## Status

Early MVP. The phone answers pings **while the app is open**. Waking a closed app needs push notifications. See [the roadmap](docs/roadmap.md).

## Repo layout

| Path | What |
|------|------|
| [`server/`](server) | Rust relay (axum). Agents call it over HTTP or MCP; the phone holds a WebSocket to it. |
| [`app/`](app) | Tauri v2 phone app (Android / iOS). Signs in, then answers pings with a GPS fix. |
| [`supabase/`](supabase) | Database migrations for agent keys. |
| [`infra/`](infra) | OpenTofu for the dev and prod environments. |
| [`scripts/`](scripts) | Local dev, end-to-end check, deploy, hook install. |
| [`docs/`](docs) | Architecture, protocol, prior art, roadmap. |

## Quick start (local)

1. Start Supabase and the relay locally. The relay runs in the foreground, so keep this terminal open:

   ```bash
   scripts/dev-up.sh
   ```

   It writes `server/.env.local` (gitignored) and prints the `app/.env.local` lines. The relay listens on `WB_PORT` (default 8787).

2. In a second terminal, run the phone app in the Android emulator. Copy the template, then fill in the values that `dev-up.sh` prints:

   ```bash
   cp app/.env.example app/.env.local
   cd app && npm install && npm run tauri android dev
   ```

   Sign in with the dev email and password form.

3. Check the whole path, from agent key to phone and back. Run it in the second terminal. The stack and the relay must already run. It needs `curl`, `jq`, `openssl`, `node` and `npm`. It prints PASS or FAIL for 12 checks and exits 1 on failure. For another port, set the same value: `WB_PORT=8788 scripts/e2e-local.sh`:

   ```bash
   scripts/e2e-local.sh
   ```

## Use it from your agent

1. In the app, create an agent key. The key starts with `wb_`. The app shows it once.
2. Add the relay as an MCP server in Claude Code:

   ```bash
   claude mcp add --transport http whereabouts <relay-url>/mcp --header "Authorization: Bearer wb_..."
   ```

   The tool is `get_phone_location`. Details: [protocol](docs/protocol.md).

3. Or call the relay directly:

   ```bash
   curl -X POST -H "Authorization: Bearer wb_..." "<relay-url>/v1/locate?timeout_secs=30"
   ```

For a local relay, `<relay-url>` is `http://127.0.0.1:8787`.

## Docs

- [Architecture](docs/architecture.md): components, sign-in and ping flow, security, limits
- [Protocol](docs/protocol.md): HTTP, MCP and WebSocket at intent level
- [Prior art](docs/prior-art.md): what exists and why this project
- [Roadmap](docs/roadmap.md)
- [Infrastructure](infra/README.md): deploy steps per environment, and [migrations](infra/README.md#migrations) (applied by GitHub Actions)

## Contributing

- Run `scripts/install-hooks.sh` once. The pre-push hook runs `linecheck .` (install with `cargo install linecheck`).
- Every file stays at 100 lines or fewer. Split a file by its job when it grows.

## License

[MIT](LICENSE)
