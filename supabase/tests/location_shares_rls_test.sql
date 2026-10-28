-- RLS: owner creates/lists/revokes own shares; grantee reads active ones by id or verified email;
-- strangers see nothing; agent tokens read as grantee but never write.
begin;
create extension if not exists pgtap with schema extensions;
select plan(13);

insert into auth.users (id, email, email_confirmed_at) values
  ('00000000-0000-0000-0000-00000000000a', 'a@test.local', now()),
  ('00000000-0000-0000-0000-00000000000b', 'b@test.local', now()),
  ('00000000-0000-0000-0000-00000000000c', 'c@test.local', now());
insert into public.location_shares (owner_user_id, grantee_email, grantee_user_id, owner_display_name) values
  ('00000000-0000-0000-0000-00000000000c', 'b@test.local', '00000000-0000-0000-0000-00000000000b', 'Cee');

set local role authenticated;
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000a","role":"authenticated","email":"a@test.local"}', true);
select lives_ok(
  $$insert into public.location_shares (grantee_email, owner_display_name) values ('b@test.local', 'Ay')$$,
  'owner shares with an email (owner_user_id defaults to auth.uid())'
);
select throws_ok(
  $$insert into public.location_shares (grantee_email, owner_display_name) values ('b@test.local', 'Ay')$$,
  '23505', null, 'duplicate active share is rejected'
);
select throws_ok(
  $$insert into public.location_shares (grantee_email, owner_display_name) values ('B@Test.local', 'Ay')$$,
  '23514', null, 'upper-case email is rejected'
);
select throws_ok(
  $$insert into public.location_shares (owner_user_id, grantee_email, owner_display_name)
    values ('00000000-0000-0000-0000-00000000000b', 'x@test.local', 'Fake')$$,
  '42501', null, 'owner cannot create a share for someone else'
);
select is((select count(*)::int from public.location_shares), 1, 'owner sees only own shares');

-- Grantee B sees C's share by id and A's by email (not yet claimed).
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000b","role":"authenticated","email":"b@test.local"}', true);
select is((select count(*)::int from public.location_shares), 2, 'grantee sees shares by email and by id');
with u as (update public.location_shares set revoked_at = now() returning 1) select is(count(*)::int, 0, 'grantee cannot revoke (RLS hides write)') from u;

-- Agent token of B: reads, never writes.
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000b","role":"authenticated","email":"b@test.local","client_id":"b-agent"}', true);
select is((select count(*)::int from public.location_shares), 2, 'grantee agent token can SELECT');
select throws_ok(
  $$insert into public.location_shares (grantee_email, owner_display_name) values ('z@test.local', 'Bee')$$,
  '42501', null, 'agent token cannot insert'
);

-- Stranger.
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-0000000000f0","role":"authenticated","email":"nobody@test.local"}', true);
select is((select count(*)::int from public.location_shares), 0, 'stranger sees nothing');

-- Owner A revokes; the grantee then sees only C's share. An owner agent token cannot revoke.
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000a","role":"authenticated","email":"a@test.local","client_id":"a-agent"}', true);
with u as (update public.location_shares set revoked_at = now() returning 1) select is(count(*)::int, 0, 'owner agent token cannot revoke') from u;
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000a","role":"authenticated","email":"a@test.local"}', true);
with u as (update public.location_shares set revoked_at = now() returning 1) select is(count(*)::int, 1, 'owner revokes own share') from u;
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000b","role":"authenticated","email":"b@test.local"}', true);
select is((select count(*)::int from public.location_shares), 1, 'revoked share is invisible to the grantee');

select * from finish();
rollback;
