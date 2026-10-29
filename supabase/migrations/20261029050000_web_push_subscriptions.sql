-- Web Push subscriptions of browser/PWA installs. The relay reads and prunes them with the agent's
-- user JWT (RLS), or reads the owner's through a share-checked RPC for shared requests.

create table public.web_push_subscriptions (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null default auth.uid() references auth.users (id) on delete cascade,
  -- Null until the install's first hello creates its devices row.
  device_id uuid references public.devices (id) on delete cascade,
  endpoint text not null unique check (endpoint like 'https://%' and char_length(endpoint) <= 2048),
  p256dh text not null check (char_length(p256dh) between 1 and 256),
  auth text not null check (char_length(auth) between 1 and 64),
  created_at timestamptz not null default now()
);

create index web_push_subscriptions_user_id_idx on public.web_push_subscriptions (user_id);

alter table public.web_push_subscriptions enable row level security;

revoke all on public.web_push_subscriptions from public, anon, authenticated;
grant select, insert, update, delete on public.web_push_subscriptions to authenticated;

create policy web_push_subscriptions_select_own on public.web_push_subscriptions
  for select to authenticated using (user_id = (select auth.uid()));
create policy web_push_subscriptions_insert_own on public.web_push_subscriptions
  for insert to authenticated with check (user_id = (select auth.uid()));
create policy web_push_subscriptions_update_own on public.web_push_subscriptions
  for update to authenticated
  using (user_id = (select auth.uid())) with check (user_id = (select auth.uid()));
create policy web_push_subscriptions_delete_own on public.web_push_subscriptions
  for delete to authenticated using (user_id = (select auth.uid()));

-- Same share check as share_target_push_tokens: an ACTIVE share with the caller as grantee.
create function public.share_target_web_push_subscriptions(p_owner uuid)
returns table (endpoint text, p256dh text, auth text)
language sql stable security definer set search_path = ''
as $$
  select w.endpoint, w.p256dh, w.auth
  from public.web_push_subscriptions w left join public.devices d on d.id = w.device_id
  where w.user_id = p_owner and (d.id is null or d.revoked_at is null)
    and exists (select 1 from public.location_shares s
      where s.owner_user_id = p_owner and s.revoked_at is null
        and (s.grantee_user_id = auth.uid() or s.grantee_email = lower(auth.jwt() ->> 'email')))
$$;

revoke all on function public.share_target_web_push_subscriptions(uuid) from public, anon;
grant execute on function public.share_target_web_push_subscriptions(uuid) to authenticated;
