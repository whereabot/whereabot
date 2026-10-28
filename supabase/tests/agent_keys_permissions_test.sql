-- Permissions: only service_role may resolve a key hash to a user.
begin;
create extension if not exists pgtap with schema extensions;
select plan(4);

insert into auth.users (id, email) values
  ('00000000-0000-0000-0000-00000000000a', 'a@test.local');

set local role authenticated;
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000a","role":"authenticated"}', true);
select set_config('test.key_a', public.create_agent_key('laptop'), true);
reset role;

select set_config('test.hash_a', encode(extensions.digest(current_setting('test.key_a'), 'sha256'), 'hex'), true);
select is(
  (select key_hash from public.agent_keys),
  current_setting('test.hash_a'),
  'stored key_hash is SHA-256 hex of the plaintext key'
);

set local role service_role;
select is(
  public.agent_key_user(current_setting('test.hash_a')),
  '00000000-0000-0000-0000-00000000000a'::uuid,
  'service_role resolves key hash to user A'
);
reset role;

set local role anon;
select throws_ok(
  $$select public.agent_key_user('00')$$,
  '42501',
  'permission denied for function agent_key_user',
  'anon cannot execute agent_key_user'
);
reset role;

set local role authenticated;
select throws_ok(
  $$select public.agent_key_user('00')$$,
  '42501',
  'permission denied for function agent_key_user',
  'authenticated cannot execute agent_key_user'
);
reset role;

select * from finish();
rollback;
