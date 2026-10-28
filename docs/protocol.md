# Protocol

The source of truth is the code: [`server/src/lib.rs`](../server/src/lib.rs) (relay) and [`app/src/protocol.ts`](../app/src/protocol.ts) (phone). This page explains intent.

## Agent → relay (HTTP)

`POST /v1/locate?timeout_secs=30` with `Authorization: Bearer <agent token>`.

| Status | Meaning |
|--------|---------|
| `200` | JSON location (below). |
| `401` | Wrong or missing agent token. |
| `503` | No phone connected. |
| `504` | The phone did not answer before the timeout (max 120 s, default 30 s). |

```json
{
  "id": "6f1c…",
  "latitude": 32.08,
  "longitude": 34.78,
  "accuracy_m": 5.0,
  "captured_at_ms": 1793180000000
}
```

`captured_at_ms` is the phone's fix time (Unix ms). Check it: an OS can return a cached fix.

## Phone ↔ relay (WebSocket)

The phone connects to `GET /v1/device` with `Authorization: Bearer <device token>`. A new connection replaces the old one.

Relay → phone:

```json
{ "type": "locate", "id": "6f1c…" }
```

Phone → relay — same `id`, so the relay can match the answer to the waiting agent:

```json
{ "type": "location", "id": "6f1c…", "latitude": 32.08, "longitude": 34.78, "accuracy_m": 5.0, "captured_at_ms": 1793180000000 }
```

The relay ignores messages it does not understand and answers with an unknown `id`.
