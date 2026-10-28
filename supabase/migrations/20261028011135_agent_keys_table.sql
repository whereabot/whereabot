-- Agent API keys for the relay. Plaintext keys are returned once by
-- create_agent_key and never stored; only the SHA-256 hex digest is kept.

create extension if not exists pgcrypto with schema extensions;

create table public.agent_keys (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null references auth.users (id) on delete cascade,
  name text not null check (char_length(name) between 1 and 64),
  prefix text not null,
  key_hash text not null unique,
  created_at timestamptz default now(),
  last_used_at timestamptz,
  revoked_at timestamptz
);

create index agent_keys_user_id_idx on public.agent_keys (user_id);

alter table public.agent_keys enable row level security;

-- Writes go only through the RPCs in the next migration.
revoke all on public.agent_keys from public, anon, authenticated;
grant select on public.agent_keys to authenticated;

create policy agent_keys_select_own
  on public.agent_keys
  for select
  to authenticated
  using (user_id = (select auth.uid()));
