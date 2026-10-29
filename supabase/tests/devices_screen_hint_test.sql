-- screen_hint: the owner writes it; a reinstall's takeover query sees only own rows.
begin;
create extension if not exists pgtap with schema extensions;
select plan(4);

insert into auth.users (id, email) values
  ('00000000-0000-0000-0000-00000000000a', 'a@test.local'),
  ('00000000-0000-0000-0000-00000000000b', 'b@test.local');
insert into public.devices (id, user_id, name, platform, app_kind) values
  ('00000000-0000-0000-0000-0000000000d1', '00000000-0000-0000-0000-00000000000a', 'iPhone', 'ios', 'pwa');

set local role authenticated;
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000a","role":"authenticated"}', true);
select lives_ok(
  $$update public.devices set screen_hint = '390x844@3' where id = '00000000-0000-0000-0000-0000000000d1'$$,
  'owner writes the hint'
);
select throws_ok(
  $$update public.devices set screen_hint = repeat('x', 33) where id = '00000000-0000-0000-0000-0000000000d1'$$,
  '23514', null, 'over-long hint is rejected'
);

select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000b","role":"authenticated"}', true);
select is(
  (select count(*)::int from public.devices where platform = 'ios' and app_kind = 'pwa'),
  0, 'user B takeover query finds no row of A'
);
reset role;
select is(
  (select screen_hint from public.devices where id = '00000000-0000-0000-0000-0000000000d1'),
  '390x844@3', 'hint stored'
);

select * from finish();
rollback;
