# Privacy

What Whereabot stores, for how long, and who can see your location. It applies to every relay that runs this code, including the hosted one at whereabot.io. The code is the source of truth: [`supabase/migrations/`](../supabase/migrations) for tables, [`server/src/`](../server/src) for what the relay logs.

## Your location

- The phone reads its location only when an agent asks, and only after you allow it, or after you chose Always allow for that agent ([Always allow](always-allow.md)). The app has no background tracking mode.
- The relay passes the fix to the agent that asked and then forgets it. It writes no location to a database or a log. See [architecture, security](architecture.md#security).
- The agent that asked gets the fix. What the agent does next (for example, sending it to its AI model provider) is outside Whereabot. Read the privacy terms of your agent.
- A person you share with can make their agent ask for your location. You approve each request, unless you chose Always allow for that person's agent. See [location sharing](location-sharing.md). The relay limits how often one agent can ask you ([rate limits](architecture.md#security)).

## What the database stores

Supabase holds sign-in and settings data. Row-level security lets each user read only their own rows.

| Data | Why |
|------|-----|
| Your account: email, name and avatar from Google, sign-in times | Sign-in (Supabase Auth) |
| `devices`: device name, platform, app kind, coarse screen size, last seen time | Route a ping to your phones; list them in the app |
| `push_tokens`, `web_push_subscriptions` | Wake the app when a request arrives |
| `always_allowed_agents`: agent client id, and the requester for a shared request | Answer an always-allowed agent without a tap |
| `location_shares`: grantee email, your display name, revoke time | Location sharing between people |
| `agent_keys`: name, prefix and a hash of each agent key | Headless agent access. The plaintext key is never stored. |

The request history in the app (which agent asked, when, and your answer) stays on the phone. It is not sent to the relay.

## What the relay logs

The relay logs operational events: user id, device id and name, app version, request id, and the outcome of each request (answered, denied, unavailable, status code). It never logs coordinates, tokens or keys. Log retention is the setting of the host that runs the relay. The hosted relay uses the Google Cloud Logging default of 30 days.

## Retention and deletion

- Rows stay until you remove them in the app or delete your account. Removing a device, revoking a share or revoking an agent key marks the row as revoked.
- Deleting the Supabase user deletes their devices, push tokens, always-allowed agents, agent keys and the shares they own (`on delete cascade`).
- Self-service account deletion in the app is not built yet ([roadmap](roadmap.md)). On the hosted service, open a [private contact request](https://github.com/whereabot/whereabot/issues/new?template=private-contact.yml) that contains no personal data. A maintainer follows up privately.

## Services involved

| Service | Role |
|---------|------|
| Supabase | Accounts and the tables above |
| Google Cloud Run | Runs the relay |
| Apple Push Notification service, Web Push services | Deliver wake-up notifications. A push has the agent name and, for a shared request, the requester's name and email. Never a location. |
| Cloudflare | Serves the landing page, docs and OAuth consent page |

When you self-host, you choose these services, and this page describes your deployment.
