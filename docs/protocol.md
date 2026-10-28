# Protocol

The code is the source of truth. This page explains intent. Exact shapes live in code:

- Relay routes: [`server/src/lib.rs`](../server/src/lib.rs)
- Messages between relay and phone: [`server/src/relay_wire_protocol.rs`](../server/src/relay_wire_protocol.rs) and [`app/src/relay-wire-protocol.ts`](../app/src/relay-wire-protocol.ts)
- MCP tool: [`server/src/mcp_location_tool.rs`](../server/src/mcp_location_tool.rs)
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
| `503` | No device is connected for this user. | The user's phone is not connected. Ask the user to open the Whereabot app, signed in as EMAIL and keep it open, then retry. |
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

A user can have several devices connected at once, each named by its `device_id` (see Devices below). A new connection with the same `device_id` replaces that device only: the relay closes the old socket with Close `4000 "replaced"`. The app must not reconnect on its own after that. On SIGTERM the relay closes every device socket with Close `1012 "restart"`; the app reconnects at once. The relay sends keep-alive pings, so idle sockets survive mobile networks.

- **Phone to relay, first:** `{"type":"hello","app_version","device_id","device_name","platform"}`. `device_id` is the install's uuid, `platform` is `ios`, `android` or `desktop`. Until `hello` arrives the socket has no identity; an old app that sends no `device_id` still works, but is never replaced by its own reconnect.
- **Relay to phone:** `{"type":"locate","id","client_id","expires_ms"}`. `client_id` comes from the token (agent keys use `agent-key`). The relay sends no name: the app maps `client_id` to the client name with `supabase.auth.oauth.listGrants()`. `expires_ms` is the epoch time when the relay stops waiting; the app aborts the prompt then. The relay resends the ping to a new socket after a reconnect, so the app handles each id once.
- **Phone to relay, answers** (all carry the ping `id`):
  - `location`: the fix.
  - `denied`, optional `reason`: `user` (default) or `no_answer`.
  - `unavailable`, `reason`: `permission`, `no_fix` or `error`.
- **Consent:** the app asks the user (Reject, Approve, Always approve) before it answers. Always-approved agents get an answer at once. Reject sends `denied`; no tap sends `denied` with `no_answer`.
- **Unknown messages:** the relay ignores them. An answer with an unknown id is dropped. An unknown `reason` counts as `user` (denied) or `error` (unavailable).

## Devices

Each install has a stable random `device_id` (uuid, kept across sign-out). It is the primary key of the `devices` table ([migration](../supabase/migrations/20261028030000_devices.sql)); the app inserts its own row on first connect, then refreshes `last_seen_at` every 60 s while connected. A device counts as online when `last_seen_at` is under 90 s old. Rename and Remove (sets `revoked_at`) are plain updates under row-level security.

- **Fan-out:** a locate request goes to every connected device of the user, and to each device that connects before it expires. The first answer (`location`, `denied` or `unavailable`) settles it. The relay then sends `{"type":"cancel","id"}` to every other device that got the ping; the app closes that prompt and records "answered on another device". The relay also sends `cancel` when the request times out. Offline (`503`) means no device is connected.
- **Revocation:** on `hello` the relay reads the device row with the phone's own bearer (PostgREST, `apikey` = `SUPABASE_ANON_KEY` when set). If `revoked_at` is set, it closes the socket with Close `4000 "revoked"`. The app then forgets its `device_id` and signs out. An unreachable Supabase counts as not revoked. A device removed while connected is closed at its next connect (or when its own heartbeat sees the row).

## Push wake-up

iOS suspends the app in the background, so a waiting request needs a nudge. When a request starts and no phone answers within about 2 s (or none is connected), the relay sends one APNs alert push per request to each token in `push_tokens`. Retries and reconnect resends send no more pushes.

- **Alert:** title `Whereabot`, body `Your agent wants to know your location`, sound default, `thread-id`, and custom key `request_id`. `apns-push-type` is `alert`, priority `10`, and `apns-expiration` is the request deadline.
- **Tap:** opens the app. The app reconnects on return to the foreground, and the relay resends the waiting `locate` to the new socket. The app needs no handling for `request_id`.
- **Tokens:** the relay reads them with the agent's own user JWT (RLS limits it to that user's rows). It deletes a token when APNs answers `410` or `BadDeviceToken`.
- **Off:** without the `APNS_*` settings the relay logs once and sends no push.
