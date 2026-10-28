-- APNs device tokens. The relay reads and prunes them with the agent's user JWT, so RLS is the only gate.

create table public.push_tokens (
  user_id uuid not null default auth.uid() references auth.users (id) on delete cascade,
  token text not null check (char_length(token) between 1 and 512),
  platform text not null default 'ios',
  updated_at timestamptz not null default now(),
  -- Per-install id; the token goes away with its device.
  device_id uuid references public.devices (id) on delete cascade,
  primary key (user_id, token)
);

alter table public.push_tokens enable row level security;

revoke all on public.push_tokens from public, anon, authenticated;
grant select, insert, update, delete on public.push_tokens to authenticated;

create policy push_tokens_select_own on public.push_tokens
  for select to authenticated using (user_id = (select auth.uid()));
create policy push_tokens_insert_own on public.push_tokens
  for insert to authenticated with check (user_id = (select auth.uid()));
create policy push_tokens_update_own on public.push_tokens
  for update to authenticated
  using (user_id = (select auth.uid())) with check (user_id = (select auth.uid()));
create policy push_tokens_delete_own on public.push_tokens
  for delete to authenticated using (user_id = (select auth.uid()));
