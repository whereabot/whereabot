-- "Always allow" on a shared request is scoped to the requester: requester_user_id null = the owner's
-- own agent (as before); set = that requester's agent. The old (user_id, client_id) key cannot hold both.

alter table public.always_allowed_agents drop constraint always_allowed_agents_pkey;
alter table public.always_allowed_agents add column id uuid not null default gen_random_uuid() primary key;
alter table public.always_allowed_agents
  add column requester_user_id uuid references auth.users (id) on delete cascade;
alter table public.always_allowed_agents
  add constraint always_allowed_agents_owner_client_requester_key
  unique nulls not distinct (user_id, client_id, requester_user_id);

-- Owners write only requester rows for someone they actively share with. Agent tokens still cannot write.
drop policy always_allowed_agents_insert_own on public.always_allowed_agents;
drop policy always_allowed_agents_update_own on public.always_allowed_agents;
create policy always_allowed_agents_insert_own on public.always_allowed_agents
  for insert to authenticated
  with check (
    user_id = (select auth.uid()) and (select auth.jwt() ->> 'client_id') is null
    and (requester_user_id is null or exists (
      select 1 from public.location_shares s
      where s.owner_user_id = user_id and s.grantee_user_id = requester_user_id and s.revoked_at is null))
  );
create policy always_allowed_agents_update_own on public.always_allowed_agents
  for update to authenticated
  using (user_id = (select auth.uid()) and (select auth.jwt() ->> 'client_id') is null)
  with check (
    user_id = (select auth.uid()) and (select auth.jwt() ->> 'client_id') is null
    and (requester_user_id is null or exists (
      select 1 from public.location_shares s
      where s.owner_user_id = user_id and s.grantee_user_id = requester_user_id and s.revoked_at is null))
  );
