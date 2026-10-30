# Whereabot

Let your AI agent ask **your phone** where you are, on demand. You host the relay. Sign-in and user data live in Supabase.

```mermaid
flowchart LR
  Agent["AI agent"] -->|"MCP or HTTP"| Relay["Relay (Rust)"]
  Relay -->|WebSocket| Phone["Phone app (Tauri)"]
  Phone -->|"GPS fix"| Relay
  Relay -->|"answer"| Agent
```

The relay keeps no location history. It sends a ping, the phone reads its GPS now, and the answer goes back to the agent. What is stored, for how long, and who sees your location: [privacy](docs/privacy.md).

## Status

Early MVP, in beta. iOS answers in the background through push. Android answers only while the app is open. See [the roadmap](docs/roadmap.md).

## Repo layout

| Path | What |
|------|------|
| [`server/`](server) | Rust relay (axum). Agents call it over HTTP or MCP; the phone holds a WebSocket to it. |
| [`app/`](app) | Tauri v2 phone app (Android / iOS). Signs in, then answers pings with a GPS fix. |
| [`supabase/`](supabase) | Database migrations and tests. |
| [`infra/`](infra) | OpenTofu for the dev and prod environments. |
| [`scripts/`](scripts) | Local dev, end-to-end check, deploy, hook install. |
| [`docs/`](docs) | Architecture, protocol, prior art, roadmap, contributing. |

## Quick start (local)

1. Start Supabase and the relay locally. The relay runs in the foreground, so keep this terminal open:

   ```bash
   scripts/dev-up.sh
   ```

   It writes `server/.env.local` (gitignored) and prints the `app/.env.local` lines.

2. In a second terminal, run the phone app in the Android emulator. Copy the template, then fill in the values that `dev-up.sh` prints:

   ```bash
   cp app/.env.example app/.env.local
   cd app && npm install && npm run tauri android dev
   ```

   Sign in with the dev email and password form.

3. Check the whole path, from agent to phone and back. Run it in the second terminal while the stack and relay run. It needs `curl`, `jq`, `openssl`, `node` and `npm`. It prints PASS or FAIL per check and exits 1 on failure. For another port, set the same value for both: `WB_PORT=8788 scripts/e2e-local.sh`:

   ```bash
   scripts/e2e-local.sh
   ```

## Connect your agent

1. Install the app: see [Get started on whereabot.io](https://whereabot.io/#start), or [install from the browser](docs/install-from-browser.md).
2. Sign in with Google on the phone.
3. Add the relay to your agent:

   ```bash
   claude mcp add --transport http whereabot https://relay.whereabot.io/mcp
   ```

4. In Claude Code run `/mcp` and sign in with the **same Google account** as on the phone. You never paste a token.
5. On Android, keep the app open while the agent asks, until Android push ships.

Relay URLs: [live URLs](infra/README.md#live-urls). The tool and its arguments: [protocol](docs/protocol.md).

## Self-host

Run your own relay and Supabase project with the OpenTofu code in [`infra/`](infra). Steps: [infra/README.md](infra/README.md). Build the app with your Supabase and relay values ([`app/.env.example`](app/.env.example)), and add your relay to your agent.

## Docs

- [Architecture](docs/architecture.md): components, sign-in and ping flow, security, limits
- [Protocol](docs/protocol.md): HTTP, MCP and WebSocket at intent level
- [Prior art](docs/prior-art.md): what exists and why this project
- [Privacy](docs/privacy.md): what is stored, retention, who sees your location
- [Roadmap](docs/roadmap.md)
- [Docs site](https://whereabot.io/docs/): built from these files by `docs-site/`, plus generated MCP and wire references
- [Infrastructure](infra/README.md): live URLs, deploy steps, migrations

## Community

- [Contributing](docs/contributing.md): hooks, line limit, adding a migration
- [Security policy](SECURITY.md): report vulnerabilities privately
- [Code of conduct](CODE_OF_CONDUCT.md)

## License

[MIT](LICENSE). Third-party material: [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
