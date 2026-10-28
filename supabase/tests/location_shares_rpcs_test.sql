-- claim_location_shares binds only the caller's verified email; the read RPCs refuse without an active share.
begin;
create extension if not exists pgtap with schema extensions;
select plan(11);

insert into auth.users (id, email, email_confirmed_at) values
  ('00000000-0000-0000-0000-00000000000a', 'a@test.local', now()),
  ('00000000-0000-0000-0000-00000000000b', 'b@test.local', now()),
  ('00000000-0000-0000-0000-00000000000d', 'd@test.local', null);
insert into public.devices (id, user_id, name, platform) values
  ('00000000-0000-0000-0000-0000000000d1', '00000000-0000-0000-0000-00000000000a', 'iPhone', 'ios');
insert into public.push_tokens (user_id, token, device_id) values
  ('00000000-0000-0000-0000-00000000000a', 'tok-a', '00000000-0000-0000-0000-0000000000d1');
insert into public.location_shares (owner_user_id, owner_email, grantee_email, owner_display_name) values
  ('00000000-0000-0000-0000-00000000000a', 'a@test.local', 'b@test.local', 'Ay'),
  ('00000000-0000-0000-0000-00000000000a', 'a@test.local', 'd@test.local', 'Ay'),
  ('00000000-0000-0000-0000-00000000000a', 'a@test.local', 'other@test.local', 'Ay');
insert into public.always_allowed_agents (user_id, client_id, requester_user_id) values
  ('00000000-0000-0000-0000-00000000000a', 'b-agent', '00000000-0000-0000-0000-00000000000b');

set local role authenticated;
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000b","role":"authenticated","email":"b@test.local"}', true);
select is((select count(*)::int from public.share_target_push_tokens('00000000-0000-0000-0000-00000000000a')), 1, 'verified email matches the share even before claim');
select is(public.claim_location_shares(), 1, 'B claims exactly the share for its verified email');
select is(public.claim_location_shares(), 0, 'claiming again is a no-op');
select is((select count(*)::int from public.share_target_push_tokens('00000000-0000-0000-0000-00000000000a')), 1, 'grantee reads owner push tokens');
select is((select owner_email from public.location_shares where grantee_user_id = auth.uid()), 'a@test.local', 'grantee reads owner_email');
select ok(public.is_always_allowed_for_requester('00000000-0000-0000-0000-00000000000a', 'b-agent'), 'always-allowed row found for the requester');
select ok(not public.is_always_allowed_for_requester('00000000-0000-0000-0000-00000000000a', 'other-agent'), 'other client is not always-allowed');

-- Unverified email cannot claim.
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000d","role":"authenticated","email":"d@test.local"}', true);
select is(public.claim_location_shares(), 0, 'unverified email claims nothing');

-- Owner revokes: the grantee RPCs now refuse.
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000a","role":"authenticated","email":"a@test.local"}', true);
update public.location_shares set revoked_at = now() where grantee_email = 'b@test.local';
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000b","role":"authenticated","email":"b@test.local"}', true);
select is((select count(*)::int from public.share_target_push_tokens('00000000-0000-0000-0000-00000000000a')), 0, 'revoked share: no push tokens');
select ok(not public.is_always_allowed_for_requester('00000000-0000-0000-0000-00000000000a', 'b-agent')
  and not exists (select 1 from public.location_shares), 'revoked share: no always-allowed, no listing');

set local role anon;
select throws_ok($$select public.claim_location_shares()$$, '42501', null, 'anon cannot call claim');

select * from finish();
rollback;
