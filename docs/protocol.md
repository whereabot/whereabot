# Protocol

The code is the source of truth. This page explains intent. Exact shapes live in code:

- Relay routes: [`server/src/lib.rs`](../server/src/lib.rs)
- Messages between relay and phone: [`server/src/protocol.rs`](../server/src/protocol.rs) and [`app/src/protocol.ts`](../app/src/protocol.ts)
- MCP tool: [`server/src/mcp.rs`](../server/src/mcp.rs)
- Credential checks: [`supabase/migrations/`](../supabase/migrations)

## Agent to relay

Target: OAuth 2.1, as in the MCP authorization spec. The MCP client signs in through the browser and sends a short-lived access token as a bearer token. Not live yet: see [roadmap](roadmap.md). The relay also accepts agent keys as a headless fallback (below).

Every agent call sends a bearer credential: `Authorization: Bearer <token>`. An unknown, expired or revoked credential gets `401`.

### HTTP: `POST /v1/locate`

Pings the user's phone and waits for the fix. Query: `timeout_secs`, default 30, capped at 120.

| Status | Meaning |
|--------|---------|
| `200` | The fix. The body is a JSON location. |
| `401` | Missing, unknown, expired or revoked credential. |
| `403` | The user declined to share their location. |
| `503` | No phone is connected for this user. |
| `504` | The phone did not answer before the timeout. |

Check the fix time. An OS can return a cached position. The fix carries the phone's capture time.

### MCP: `/mcp`

Streamable HTTP, with JSON responses and no session. The same bearer header gates it.

- One tool: `get_phone_location`. Optional `timeout_secs`, 1 to 120, default 30.
- A fix returns as text: the JSON location.
- Offline, timeout or a declined request returns a tool error with a short message. The HTTP status stays `200`.
- A bad credential returns `401` before any tool runs.

## Agent keys (headless fallback)

For headless or CI use only. Users do not onboard agents this way.

- **Create:** in the app. The key starts with `wb_`. The app shows it once.
- **Rotate:** create a new key, update the headless config, then revoke the old key in the app.
- **Leak:** revoke the key in the app. A revoked key gets `401`.

## Phone to relay

The phone opens a WebSocket to `/v1/device` with its Supabase access token as a bearer token. A bad token gets `401` before the upgrade.

A new connection replaces the user's old one. The relay sends keep-alive pings, so idle sockets survive mobile networks.

- **Relay to phone:** a ping with a new id and the asking agent's `client_id` (from the token; agent keys use `agent-key`). The relay sends no name: the app maps `client_id` to the client name with `supabase.auth.oauth.listGrants()`. The relay resends the ping to a new socket after a reconnect, so the app handles each id once.
- **Phone to relay:** a fix with the same id, so the relay matches it to the waiting agent.
- **Consent:** the app asks the user (Reject, Approve, Always approve) before it answers. Always-approved agents get an answer at once. Reject sends `denied` with the id, and the relay ends the request: MCP tool error `The user declined to share their location.`, HTTP `403`. No answer in about 55 s sends nothing, and the agent times out.
- **Unknown messages:** the relay ignores them. A fix with an unknown id is dropped.
