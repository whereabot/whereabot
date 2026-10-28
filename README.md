# Whereabouts

Let your AI agent ask **your phone** where you are — on demand, through a relay you host yourself.

```
agent ──HTTP──▶ relay (Rust) ──WebSocket──▶ phone app (Tauri) ──GPS fix──▶ relay ──▶ agent
```

The agent never sees a stored location history. It sends a ping, the phone reads its GPS right now, and the answer goes back to the agent.

## Status

Early MVP. The phone answers pings **while the app is open**. Waking a closed app needs push notifications (APNs / FCM) — see [docs/roadmap.md](docs/roadmap.md).

## Repo layout

| Path | What |
|------|------|
| [`server/`](server) | Rust relay (axum). Agents call it over HTTP; the phone holds a WebSocket to it. |
| [`app/`](app) | Tauri v2 phone app (Android / iOS). Answers pings with a GPS fix. |
| [`docs/`](docs) | Architecture, wire protocol, prior art, roadmap. |

## Quick start

```bash
# 1. Run the relay
cd server
WHEREABOUTS_AGENT_TOKEN=$(openssl rand -hex 32) \
WHEREABOUTS_DEVICE_TOKEN=$(openssl rand -hex 32) \
cargo run

# 2. Run the phone app (needs Android SDK or Xcode)
cd app && npm install
npm run tauri android init && npm run tauri android dev   # or: ios init / ios dev
# After init, add location permissions (generated projects are not committed):
#   iOS:     NSLocationWhenInUseUsageDescription in gen/apple/app_iOS/Info.plist
#   Android: ACCESS_FINE_LOCATION + ACCESS_COARSE_LOCATION in gen/android/app/src/main/AndroidManifest.xml
# In the app: enter the relay URL and the device token, tap Connect.

# 3. Ask for the location as the agent
curl -X POST -H "Authorization: Bearer $WHEREABOUTS_AGENT_TOKEN" \
  "http://127.0.0.1:8787/v1/locate?timeout_secs=30"
```

Expose the relay with TLS (`wss://`) before you use it outside your LAN — see [docs/architecture.md](docs/architecture.md#security).

## Docs

- [Architecture](docs/architecture.md) — components, flow, security
- [Protocol](docs/protocol.md) — HTTP and WebSocket messages
- [Prior art](docs/prior-art.md) — what exists and why this project
- [Roadmap](docs/roadmap.md)

## License

[MIT](LICENSE)
