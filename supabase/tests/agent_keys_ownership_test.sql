-- Ownership: each user sees and revokes only their own agent keys.
begin;
create extension if not exists pgtap with schema extensions;
select plan(6);

insert into auth.users (id, email) values
  ('00000000-0000-0000-0000-00000000000a', 'a@test.local'),
  ('00000000-0000-0000-0000-00000000000b', 'b@test.local');

-- User A creates a key.
set local role authenticated;
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000a","role":"authenticated"}', true);
select set_config('test.key_a', public.create_agent_key('laptop'), true);
select set_config('test.id_a', (select id::text from public.agent_keys), true);

select matches(current_setting('test.key_a'), '^wb_[0-9a-f]{48}$', 'create_agent_key returns wb_ plus 48 lowercase hex');
select is((select count(*)::int from public.agent_keys), 1, 'user A sees exactly 1 own row');
select throws_ok(
  $$insert into public.agent_keys (user_id, name, prefix, key_hash) values (auth.uid(), 'x', 'wb_00000000', 'deadbeef')$$,
  '42501',
  'permission denied for table agent_keys',
  'authenticated cannot insert into agent_keys directly'
);

-- User B sees nothing and cannot revoke A's key.
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000b","role":"authenticated"}', true);
select is((select count(*)::int from public.agent_keys), 0, 'user B sees 0 rows');
select lives_ok(
  $$select public.revoke_agent_key(current_setting('test.id_a')::uuid)$$,
  'user B call to revoke A key runs without error'
);
reset role;
select ok(
  (select revoked_at is null from public.agent_keys where id = current_setting('test.id_a')::uuid),
  'B revoke leaves A key active'
);

select * from finish();
rollback;
