-- RLS: users see, create, rename and revoke only their own devices.
begin;
create extension if not exists pgtap with schema extensions;
select plan(8);

insert into auth.users (id, email) values
  ('00000000-0000-0000-0000-00000000000a', 'a@test.local'),
  ('00000000-0000-0000-0000-00000000000b', 'b@test.local');

insert into public.devices (id, user_id, name, platform) values
  ('00000000-0000-0000-0000-0000000000d1', '00000000-0000-0000-0000-00000000000a', 'iPhone', 'ios');

set local role authenticated;
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000a","role":"authenticated"}', true);
select is((select count(*)::int from public.devices), 1, 'user A sees own device');
select lives_ok(
  $$insert into public.devices (id, name, platform) values ('00000000-0000-0000-0000-0000000000d2', 'Mac', 'desktop')$$,
  'user A registers a device (user_id defaults to auth.uid())'
);
select lives_ok(
  $$update public.devices set name = 'Work iPhone', revoked_at = now() where id = '00000000-0000-0000-0000-0000000000d1'$$,
  'user A renames and revokes own device'
);
select throws_ok(
  $$insert into public.devices (id, name, platform) values ('00000000-0000-0000-0000-0000000000d3', 'Fax', 'fax')$$,
  '23514', null, 'unknown platform is rejected'
);

select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000b","role":"authenticated"}', true);
select is((select count(*)::int from public.devices), 0, 'user B sees no devices of A');
select throws_ok(
  $$insert into public.devices (id, user_id, name, platform) values ('00000000-0000-0000-0000-0000000000d4', '00000000-0000-0000-0000-00000000000a', 'x', 'ios')$$,
  '42501', null, 'user B cannot create a device for A'
);
select throws_ok(
  $$insert into public.devices (id, name, platform) values ('00000000-0000-0000-0000-0000000000d1', 'steal', 'ios')$$,
  '23505', null, 'user B cannot take over A device id'
);
update public.devices set name = 'hacked', revoked_at = null where id = '00000000-0000-0000-0000-0000000000d1';
reset role;
select is(
  (select name from public.devices where id = '00000000-0000-0000-0000-0000000000d1'),
  'Work iPhone',
  'user B update changed nothing'
);

select * from finish();
rollback;
