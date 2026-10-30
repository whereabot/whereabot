-- An agent token (client_id claim) cannot write devices, push tokens or web push subscriptions.
-- The user's app session still can, and the agent can still prune.
begin;
create extension if not exists pgtap with schema extensions;
select plan(7);

insert into auth.users (id, email) values ('00000000-0000-0000-0000-00000000000a', 'a@test.local');
insert into public.devices (id, user_id, name, platform, revoked_at) values
  ('00000000-0000-0000-0000-0000000000d1', '00000000-0000-0000-0000-00000000000a', 'iPhone', 'ios', now());
insert into public.push_tokens (user_id, token, device_id) values
  ('00000000-0000-0000-0000-00000000000a', 'tok-1', '00000000-0000-0000-0000-0000000000d1');

set local role authenticated;
select set_config('request.jwt.claims',
  '{"sub":"00000000-0000-0000-0000-00000000000a","role":"authenticated","client_id":"agent-1"}', true);
update public.devices set revoked_at = null where id = '00000000-0000-0000-0000-0000000000d1';
select throws_ok(
  $$insert into public.devices (id, name, platform) values ('00000000-0000-0000-0000-0000000000d2', 'x', 'ios')$$,
  '42501', null, 'agent token cannot register a device'
);
select throws_ok(
  $$insert into public.push_tokens (token) values ('agent-tok')$$,
  '42501', null, 'agent token cannot add a push token'
);
select throws_ok(
  $$insert into public.web_push_subscriptions (endpoint, p256dh, auth) values ('https://fcm.googleapis.com/x', 'k', 'a')$$,
  '42501', null, 'agent token cannot add a web push subscription'
);
select lives_ok(
  $$delete from public.push_tokens where token = 'tok-1'$$,
  'agent token can still prune push tokens'
);

select set_config('request.jwt.claims',
  '{"sub":"00000000-0000-0000-0000-00000000000a","role":"authenticated"}', true);
select is((select count(*)::int from public.push_tokens), 0, 'prune removed the token');
select lives_ok(
  $$insert into public.devices (id, name, platform) values ('00000000-0000-0000-0000-0000000000d3', 'Mac', 'desktop')$$,
  'app session registers a device'
);
reset role;
select isnt(
  (select revoked_at from public.devices where id = '00000000-0000-0000-0000-0000000000d1'),
  null, 'agent token update left the device revoked'
);

select * from finish();
rollback;
