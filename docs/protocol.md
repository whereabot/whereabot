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

Pings the user's phone and waits for the fix. Query: `timeout_secs`, default 60, capped at 120. Non-200 bodies are the agent text in the table.

| Status | Meaning | Agent text |
|--------|---------|------------|
| `200` | The fix. The body is a JSON location. | |
| `401` | Missing, unknown, expired or revoked credential. | |
| `403` | The user declined (`denied`, reason `user`). | The user declined to share their location. |
| `408` | The user did not tap in time (`denied`, reason `no_answer`). | The user did not respond on their phone. Ask them before trying again. |
| `424` | `unavailable` with reason `permission`. | Location access is off for Whereabot on the user's phone. Ask them to allow it in Settings. |
| `424` | `unavailable` with reason `no_fix` or `error`. | The phone could not get a location fix. Try again in a moment. |
| `503` | No phone is connected for this user. | The user's phone is not connected. Ask the user to open the Whereabot app, signed in as EMAIL and keep it open, then retry. |
| `504` | The phone got the ping but did not answer before the timeout. | The phone did not answer in time. Ask the user to open Whereabot and retry. |

`, signed in as EMAIL` appears only when the token has an `email` claim. It helps when the phone and the agent use different accounts.

Check the fix time. An OS can return a cached position. The fix carries the phone's capture time.

### MCP: `/mcp`

Streamable HTTP, with JSON responses and no session. The same bearer header gates it.

- One tool: `get_phone_location`. Optional `timeout_secs`, 1 to 120, default 60.
- A fix returns as text: the JSON location.
- Every other outcome returns a tool error with the agent text above. The HTTP status stays `200`.
- A bad credential returns `401` before any tool runs.

## Agent keys (headless fallback)

For headless or CI use only. Users do not onboard agents this way.

- **Create:** in the app. The key starts with `wb_`. The app shows it once.
- **Rotate:** create a new key, update the headless config, then revoke the old key in the app.
- **Leak:** revoke the key in the app. A revoked key gets `401`.

## Phone to relay

The phone opens a WebSocket to `/v1/device` with its Supabase access token as a bearer token. A bad token gets `401` before the upgrade.

A new connection replaces the user's old one: the relay closes the old socket with Close `4000 "replaced"`. The app must not reconnect on its own after that. On SIGTERM the relay closes every device socket with Close `1012 "restart"`; the app reconnects at once. The relay sends keep-alive pings, so idle sockets survive mobile networks.

- **Phone to relay, first:** `{"type":"hello","app_version":"0.1.6"}`. The relay logs it and needs nothing else.
- **Relay to phone:** `{"type":"locate","id","client_id","expires_ms"}`. `client_id` comes from the token (agent keys use `agent-key`). The relay sends no name: the app maps `client_id` to the client name with `supabase.auth.oauth.listGrants()`. `expires_ms` is the epoch time when the relay stops waiting; the app aborts the prompt then. The relay resends the ping to a new socket after a reconnect, so the app handles each id once.
- **Phone to relay, answers** (all carry the ping `id`):
  - `location`: the fix.
  - `denied`, optional `reason`: `user` (default) or `no_answer`.
  - `unavailable`, `reason`: `permission`, `no_fix` or `error`.
- **Consent:** the app asks the user (Reject, Approve, Always approve) before it answers. Always-approved agents get an answer at once. Reject sends `denied`; no tap sends `denied` with `no_answer`.
- **Unknown messages:** the relay ignores them. An answer with an unknown id is dropped. An unknown `reason` counts as `user` (denied) or `error` (unavailable).
