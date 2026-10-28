-- Location sharing: an owner shares their location with an email. The grantee's agent may then ask
-- the owner's phone (the owner still approves unless they chose "Always allow"). Owners write rows
-- from their app session only; a grantee, or an agent token of either side, can never write.

create table public.location_shares (
  id uuid primary key default gen_random_uuid(),
  owner_user_id uuid not null default auth.uid() references auth.users (id) on delete cascade,
  grantee_email text not null check (grantee_email = lower(grantee_email) and grantee_email like '%_@_%'
    and char_length(grantee_email) <= 320),
  -- Filled by claim_location_shares() when the grantee signs in with that verified email.
  grantee_user_id uuid references auth.users (id) on delete set null,
  -- The owner's verified email from their session, so the grantee's agent can match/list by email.
  owner_email text default lower(auth.jwt() ->> 'email') check (owner_email = lower(owner_email)),
  owner_display_name text not null check (char_length(owner_display_name) between 1 and 100),
  created_at timestamptz not null default now(),
  revoked_at timestamptz
);

create unique index location_shares_active_uniq
  on public.location_shares (owner_user_id, grantee_email) where revoked_at is null;
create index location_shares_grantee_user_idx on public.location_shares (grantee_user_id);
create index location_shares_grantee_email_idx on public.location_shares (grantee_email);

alter table public.location_shares enable row level security;

revoke all on public.location_shares from public, anon, authenticated;
grant select on public.location_shares to authenticated;
grant insert (grantee_email, owner_display_name) on public.location_shares to authenticated;
grant update (revoked_at) on public.location_shares to authenticated;

create policy location_shares_select_owner on public.location_shares
  for select to authenticated using (owner_user_id = (select auth.uid()));
-- Revoked shares vanish for the grantee. Agent tokens of the grantee read through this too.
create policy location_shares_select_grantee on public.location_shares
  for select to authenticated using (
    revoked_at is null
    and (grantee_user_id = (select auth.uid())
         or grantee_email = lower((select auth.jwt() ->> 'email')))
  );
create policy location_shares_insert_owner on public.location_shares
  for insert to authenticated
  with check (owner_user_id = (select auth.uid()) and (select auth.jwt() ->> 'client_id') is null);
create policy location_shares_update_owner on public.location_shares
  for update to authenticated
  using (owner_user_id = (select auth.uid()) and (select auth.jwt() ->> 'client_id') is null)
  with check (owner_user_id = (select auth.uid()) and (select auth.jwt() ->> 'client_id') is null);
