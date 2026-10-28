-- Ownership: users cannot create or revoke agent keys, and see only their own rows.
begin;
create extension if not exists pgtap with schema extensions;
select plan(6);

insert into auth.users (id, email) values
  ('00000000-0000-0000-0000-00000000000a', 'a@test.local'),
  ('00000000-0000-0000-0000-00000000000b', 'b@test.local');

-- Keys are seeded as postgres. Users cannot create them through the RPC.
insert into public.agent_keys (id, user_id, name, prefix, key_hash) values
  ('00000000-0000-0000-0000-0000000000a1', '00000000-0000-0000-0000-00000000000a', 'laptop', 'wb_aaaaaaaa', 'hash-a');

-- User A sees only own row, and cannot create or insert keys.
set local role authenticated;
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000a","role":"authenticated"}', true);
select is((select count(*)::int from public.agent_keys), 1, 'user A sees exactly 1 own row');
select throws_ok(
  $$select public.create_agent_key('laptop')$$,
  '42501',
  'permission denied for function create_agent_key',
  'authenticated cannot create agent keys'
);
select throws_ok(
  $$insert into public.agent_keys (user_id, name, prefix, key_hash) values (auth.uid(), 'x', 'wb_00000000', 'deadbeef')$$,
  '42501',
  'permission denied for table agent_keys',
  'authenticated cannot insert into agent_keys directly'
);

-- User B sees nothing and cannot revoke A's key.
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000b","role":"authenticated"}', true);
select is((select count(*)::int from public.agent_keys), 0, 'user B sees 0 rows');
select throws_ok(
  $$select public.revoke_agent_key('00000000-0000-0000-0000-0000000000a1'::uuid)$$,
  '42501',
  'permission denied for function revoke_agent_key',
  'authenticated cannot revoke agent keys'
);
reset role;
select ok(
  (select revoked_at is null from public.agent_keys where id = '00000000-0000-0000-0000-0000000000a1'::uuid),
  'failed revoke leaves A key active'
);

select * from finish();
rollback;
