-- One row per app install (phone, later the computer). The app picks `id`
-- (its device_id) and upserts name/platform/last_seen_at on connect. Removing a
-- device sets revoked_at; the relay closes revoked devices with 4000 "revoked".

create table public.devices (
  id uuid primary key,
  user_id uuid not null default auth.uid() references auth.users (id) on delete cascade,
  name text not null check (char_length(name) between 1 and 64),
  platform text not null check (platform in ('ios', 'android', 'desktop')),
  created_at timestamptz not null default now(),
  last_seen_at timestamptz not null default now(),
  revoked_at timestamptz
);

create index devices_user_id_idx on public.devices (user_id);

alter table public.devices enable row level security;

revoke all on public.devices from public, anon, authenticated;
grant select, insert, update on public.devices to authenticated;

create policy devices_select_own on public.devices
  for select to authenticated
  using (user_id = (select auth.uid()));

create policy devices_insert_own on public.devices
  for insert to authenticated
  with check (user_id = (select auth.uid()));

create policy devices_update_own on public.devices
  for update to authenticated
  using (user_id = (select auth.uid()))
  with check (user_id = (select auth.uid()));

-- Push tokens (separate migration) point at the device they belong to.
do $$
begin
  if to_regclass('public.push_tokens') is not null then
    alter table public.push_tokens
      add constraint push_tokens_device_id_fkey
      foreign key (device_id) references public.devices (id) on delete cascade;
  end if;
end $$;
