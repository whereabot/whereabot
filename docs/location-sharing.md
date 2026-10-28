# Location sharing between people

A person (the owner) shares their location with someone else's email in the app. That person's agent (the requester's agent) can then ask for the owner's location. The owner's phone asks, or answers on its own when the owner chose Always allow for that requester. Wire basics: [protocol](protocol.md). States: [request flow](request-flow.md#shared-requests).

## Agent surface

Code: [`server/src/location_target.rs`](../server/src/location_target.rs), [`server/src/shared_person_resolution.rs`](../server/src/shared_person_resolution.rs), [`server/src/list_people_tool.rs`](../server/src/list_people_tool.rs).

- `get_phone_location` takes an optional `person`: the email or display name of someone who shares with you. Case does not matter. Absent or blank = your own phones, unchanged.
- `POST /v1/locate` takes the same `person` query parameter.
- `list_people_sharing_with_me` returns `[{"display_name","email"}]`: your active shares as grantee.
- Results are the same as for your own phones, with these extra texts:

| Status | When | Agent text |
|--------|------|------------|
| `403` | The owner declined. | NAME declined to share their location. |
| `404` | No active share matches `person` (unknown, revoked, or an agent key). | Nobody named PERSON shares their location with you. Ask them to share it in the Whereabot app. |
| `409` | More than one owner matches. | More than one person named PERSON shares their location with you: NAME (EMAIL), …. Ask again with their email. |

`503` names the owner's email in `signed in as EMAIL`.

## How the relay finds the owner

The relay reads `location_shares` with the agent's own user JWT: `select=owner_user_id,owner_display_name,owner_email&revoked_at=is.null`. RLS returns only rows where the caller is the grantee. The relay never trusts a name from the agent: it pings only an `owner_user_id` that this read returned. A share of your own is dropped.

## What the owner's phones get

- **Fan-out** goes to the owner's devices. Only the owner's phones can answer: the waiting request is keyed by the owner.
- **Ping:** `locate` gains `requester: {user_id, name, email}`. It is `null` for the owner's own agent. `name` is the requester's `user_metadata.full_name` or `name` from their token, else their email.
- **Push tokens:** the requester's JWT cannot read the owner's `push_tokens`. The relay calls RPC `share_target_push_tokens(p_owner)`. It returns the owner's iOS tokens only while an active share to the caller exists. The relay does not delete dead owner tokens from a shared request; the owner's own requests do that.
- **Always allow:** RPC `is_always_allowed_for_requester(p_owner, p_client_id)` checks the share, then a row (owner, client_id, requester_user_id = caller). The owner's own lookup now filters `requester_user_id=is.null`, so a requester's row never trusts the owner's own agent.
- Both RPCs are security definer, for `authenticated` only: [`supabase/migrations/`](../supabase/migrations).

## Push copy (owner's phone)

Every push body also carries `requester` (same object, or `null`).

| Case | Body | Category |
|------|------|----------|
| Not always allowed | `REQUESTER's agent AGENT wants to know your location` (`REQUESTER's agent wants…` without a name) | `LOCATION_REQUEST` |
| Always allowed, silent push gets no answer | `REQUESTER's agent AGENT is checking your location` | `LOCATION_AUTO` |

Always allow on a shared request writes `always_allowed_agents.requester_user_id` = the requester.

## Tests

[`server/tests/location_sharing.rs`](../server/tests/location_sharing.rs), [`server/tests/location_sharing_errors.rs`](../server/tests/location_sharing_errors.rs), [`server/src/push/shared_request_push_tests.rs`](../server/src/push/shared_request_push_tests.rs).
