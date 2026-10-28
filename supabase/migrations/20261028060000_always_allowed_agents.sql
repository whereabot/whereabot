-- Agents the user chose "Always allow" for. The relay reads them with the agent's user JWT to
-- send a silent push instead of a consent alert, so RLS must stop an agent token from adding
-- itself: writes need a token without a `client_id` claim (the user's own app session).

create table public.always_allowed_agents (
  user_id uuid not null default auth.uid() references auth.users (id) on delete cascade,
  -- Generic ids (app/src/agent-identity.ts) cover many agents and are never always-allowed.
  client_id text not null
    check (char_length(client_id) between 1 and 255 and client_id not in ('unknown', 'agent-key')),
  client_name text check (char_length(client_name) <= 200),
  created_at timestamptz not null default now(),
  primary key (user_id, client_id)
);

alter table public.always_allowed_agents enable row level security;

revoke all on public.always_allowed_agents from public, anon, authenticated;
grant select, insert, update, delete on public.always_allowed_agents to authenticated;

create policy always_allowed_agents_select_own on public.always_allowed_agents
  for select to authenticated using (user_id = (select auth.uid()));
create policy always_allowed_agents_insert_own on public.always_allowed_agents
  for insert to authenticated
  with check (user_id = (select auth.uid()) and (select auth.jwt() ->> 'client_id') is null);
create policy always_allowed_agents_update_own on public.always_allowed_agents
  for update to authenticated
  using (user_id = (select auth.uid()) and (select auth.jwt() ->> 'client_id') is null)
  with check (user_id = (select auth.uid()) and (select auth.jwt() ->> 'client_id') is null);
create policy always_allowed_agents_delete_own on public.always_allowed_agents
  for delete to authenticated
  using (user_id = (select auth.uid()) and (select auth.jwt() ->> 'client_id') is null);
