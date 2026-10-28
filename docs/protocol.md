# Protocol

The code is the source of truth. This page explains intent. Exact shapes live in code:

- Relay routes: [`server/src/lib.rs`](../server/src/lib.rs)
- Messages between relay and phone: [`server/src/protocol.rs`](../server/src/protocol.rs) and [`app/src/protocol.ts`](../app/src/protocol.ts)
- MCP tool: [`server/src/mcp.rs`](../server/src/mcp.rs)
- Agent key checks: [`supabase/migrations/`](../supabase/migrations)

## Agent to relay

Every agent call sends an agent key as a bearer token: `Authorization: Bearer wb_…`. An unknown or revoked key gets `401`.

### HTTP: `POST /v1/locate`

Pings the user's phone and waits for the fix. Query: `timeout_secs`, default 30, capped at 120.

| Status | Meaning |
|--------|---------|
| `200` | The fix. The body is a JSON location. |
| `401` | Missing, unknown or revoked agent key. |
| `503` | No phone is connected for this user. |
| `504` | The phone did not answer before the timeout. |

Check the fix time. An OS can return a cached position. The fix carries the phone's capture time.

### MCP: `/mcp`

Streamable HTTP, with JSON responses and no session. The same bearer header gates it.

- One tool: `get_phone_location`. Optional `timeout_secs`, 1 to 120, default 30.
- A fix returns as text: the JSON location.
- Offline or timeout returns a tool error with a short message. The HTTP status stays `200`.
- A bad key returns `401` before any tool runs.

## Phone to relay

The phone opens a WebSocket to `/v1/device` with its Supabase access token as a bearer token. A bad token gets `401` before the upgrade.

A new connection replaces the user's old one. The relay sends keep-alive pings, so idle sockets survive mobile networks.

- **Relay to phone:** a ping with a new id. The phone answers it.
- **Phone to relay:** a fix with the same id, so the relay matches it to the waiting agent.
- **Unknown messages:** the relay ignores them. A fix with an unknown id is dropped.
