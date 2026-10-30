# Location sharing: data and RLS

An owner shares their location with an email. The grantee's agent can then ask the owner's phone, and the owner approves each time unless they chose "Always allow". How agents use it: [location sharing](location-sharing.md). The code is the source of truth: the `20261029*` files in [`supabase/migrations/`](../supabase/migrations) and the `location_shares_*` tests in [`supabase/tests/`](../supabase/tests).

## Tables

- `location_shares`: one row per owner and grantee email. Only one active (`revoked_at is null`) row per pair. Emails are lower-case.
- `always_allowed_agents.requester_user_id`: null means the owner's own agent. A value means that requester's agent is always allowed by the owner. Unique on `(user_id, client_id, requester_user_id) nulls not distinct`.

## Who may do what

- Owner (app session, no `client_id` claim): insert `grantee_email` and `owner_display_name`, select own rows, update only `revoked_at`.
- Grantee: select active rows where `grantee_user_id = auth.uid()` or `grantee_email` equals the JWT email. Never writes.
- Agent tokens (JWT with `client_id`): select as grantee or owner, never write.
- Requester rows in `always_allowed_agents` need an active share to that requester.

## RPCs (security definer, empty search_path, `authenticated` only)

- `claim_location_shares()`: binds shares to the caller's verified email (`email_confirmed_at` set). Returns the claimed count. The app calls it on sign-in.
- `share_target_push_tokens(p_owner)`: iOS push tokens of the owner. Empty without an active share to the caller.
- `is_always_allowed_for_requester(p_owner, p_client_id)`: true when the owner always-allowed that agent for the caller. False without an active share.
