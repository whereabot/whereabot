# Architecture

## Components

- **Agent**: any AI agent or script. It calls the relay over MCP or HTTP.
- **Relay** ([`server/`](../server)): a small Rust service in a container. It checks credentials, pings the phone, and passes the fix back. It holds no location history.
- **Supabase**: user sign-in (Google, and email for dev) and the agent key table. See [`supabase/`](../supabase).
- **Phone app** ([`app/`](../app)): a Tauri v2 app on Android or iOS. It signs in, keeps a WebSocket open to the relay, and answers pings with a fresh GPS fix.

Live URLs and deploy steps per environment: [infra/README.md](../infra/README.md).

## Why a relay

A phone has no public address, and an agent cannot reach it directly. The phone opens an outbound connection to the relay and keeps it open. The relay can then send a ping at any time. HTTP alone cannot do this, because only the phone can start a request.

## Sign-in and ping

```mermaid
sequenceDiagram
    participant U as User
    participant P as Phone app
    participant S as Supabase
    participant R as Relay
    participant A as Agent
    U->>P: Sign in with Google
    P->>S: OAuth (PKCE), redirect to app
    S-->>P: Session (access token)
    P->>R: Connect with access token
    R->>S: Check token, get user
    A->>R: Request with agent credential
    R->>S: Check agent credential, get user
    R->>P: Ping for that user's phone
    P->>P: Read GPS now
    P->>R: Fix with the same ping id
    R->>A: Location answer
```

The relay keeps one connection per device, and a user can have several devices. If the user has no connected device, the agent gets an offline answer. Status codes and message shapes: [protocol.md](protocol.md).

## Consent on the phone

Each ping carries the agent's OAuth client id and an `expires_ms` deadline. The phone shows a consent prompt for that agent. The user can allow once, deny, or choose always-allow for that agent. An always-allowed agent is never asked again: the phone answers and shows "<Agent> checked your location" ([Always allow](always-allow.md)). The phone then answers with one of:

- a location;
- `denied`: the user declined, or did not respond before `expires_ms`;
- `unavailable`: location permission is off, or no fix.

The relay turns each answer into a distinct agent-facing error. If the phone is offline, the error names the signed-in account, so the user can spot a mismatch with the phone. Wire format: [protocol.md](protocol.md).

## Security

- **Two credentials.** The agent credential may ask for a location. The phone access token may answer one. Neither works for the other side.
- **Isolation per user.** An agent credential maps to one user. It reaches only that user's phone. The isolation tests in [`server/tests/`](../server/tests) cover this.
- **No location storage.** A fix lives in memory for one request. Nothing writes it to a database or a log.
- **Hashed agent keys.** The database stores a hash of each agent key, never the plaintext. The plaintext is shown once. Algorithm and schema: [`supabase/migrations/`](../supabase/migrations).
- **Row-level security.** Supabase RLS limits each user to their own agent keys. Writes go through functions.
- **Rate limits.** Each locate request can alert the owner's phone. The relay limits locate requests per requester→owner pair and caps open requests per owner. Over a limit it answers 429 with `Retry-After`. Defaults and `WB_LOCATE_*` settings: [`locate_rate_limit.rs`](../server/src/locate_rate_limit.rs).
- **TLS.** Clients reach the relay over HTTPS and WSS. The container host terminates TLS.
- **No user-managed tokens.** Users never create or paste tokens. Agents sign in with OAuth. Agent keys remain a headless fallback; see [protocol](protocol.md#agent-keys-headless-fallback).

## Limits today

- **One relay instance.** Phone connections live in memory, so the relay runs as exactly one instance.
- **Answers need a tap,** except for always-allowed agents: those get a silent push and the phone answers on its own ([Always allow](always-allow.md)). iOS skips silent pushes for a force-quit app and throttles them, so a "checking" alert with Share is the fallback. Apple's location push entitlement (`com.apple.developer.location.push`) would remove that gap. Not requested.
- **Several devices per user.** A new connection replaces only the device with the same `device_id`. See [protocol](protocol.md#phone-to-relay).
- **Timeouts.** Ping waits are bounded. See [protocol](protocol.md#http-post-v1locate).

## Push wake-up

The phone stores its APNs token in `push_tokens` (Supabase, RLS: own rows only) after sign-in and notification permission, and deletes it on sign-out. If a location request finds no live phone, the relay pushes through APNs. The user taps the notification, the app opens and connects, and the waiting request reaches the consent prompt. Setup: [infra/README.md](../infra/README.md). Wire details: [protocol.md](protocol.md#push-wake-up).

## Background answer

iOS suspends the web view when the app leaves the front, so the JS prompt works only with the app open. The push therefore carries its own consent: the `LOCATION_REQUEST` category has Reject, Allow and Always allow, none with the `.foreground` option, so iOS runs the action in the background and the app stays closed.

```mermaid
sequenceDiagram
    participant A as Agent
    participant R as Relay
    participant N as APNs
    participant H as Native handler (iOS)
    participant S as Supabase
    A->>R: Locate
    R->>N: Alert, category LOCATION_REQUEST, request_id, client_id, expires_ms
    N->>H: User taps Allow on the notification
    H->>H: One CLLocationManager read
    H->>S: Refresh the access token if it is about to expire
    H->>R: POST /v1/device/answer with the fix
    R->>A: Location answer
```

- **Native code:** a local Tauri plugin, [`app/src-tauri/background-answer-plugin/`](../app/src-tauri/background-answer-plugin). Its Swift package builds with the Rust crate, so `tauri ios build` regenerating `gen/apple` does not drop it. At launch it registers the category and puts its notification delegate in front of the delegates of the push and notification plugins; everything that is not one of its three actions goes on to them. The read's accuracy steps and time limits: [`BackgroundLocationFix.swift`](../app/src-tauri/background-answer-plugin/ios/Sources/BackgroundLocationFix.swift).
- **Session:** supabase-js stores its session through the plugin in the iOS Keychain (`AfterFirstUnlock`), see [`app/src/native-session-storage.ts`](../app/src/native-session-storage.ts). The handler reads the same item and writes back a refreshed session. One refresh token for both sides, so a rotation never revokes the other side's token. Tokens are never logged.
- **Always allow:** the handler answers like Allow and saves the `client_id` to Supabase `always_allowed_agents` at once, so the next request is tapless ([Always allow](always-allow.md)). A `UserDefaults` copy lets the app retry the save on its next launch ([`app/src/always-allowed-sync.ts`](../app/src/always-allowed-sync.ts)).
- **Permission:** a read with the app closed needs location access "Always". The permissions screen asks for it after When In Use, and shows Settings steps when it is refused. Without it the handler answers `unavailable` with reason `permission`. There is no `location` background mode: the app never tracks.

Every state and notification of one request, as a diagram: [request flow](request-flow.md).
