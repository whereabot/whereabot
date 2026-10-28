-- Limits and revocation: 10 active keys per user, and revoked keys stop resolving.
-- Keys are created as postgres with a JWT claim, since users cannot call the RPC.
begin;
create extension if not exists pgtap with schema extensions;
select plan(4);

insert into auth.users (id, email) values
  ('00000000-0000-0000-0000-00000000000a', 'a@test.local');

select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000a","role":"authenticated"}', true);
select set_config('test.key_a', public.create_agent_key('laptop'), true);
select set_config('test.id_a', (select id::text from public.agent_keys), true);

select lives_ok(
  $$select count(public.create_agent_key('bulk-' || g)) from generate_series(2, 10) g$$,
  'postgres creates keys 2 to 10'
);
select throws_ok(
  $$select public.create_agent_key('eleventh')$$,
  'P0001',
  'agent key limit reached (max 10 active keys)',
  'the 11th active key raises'
);

-- Users cannot revoke. Revoke as postgres to test revocation effect.
set local role authenticated;
select throws_ok(
  $$select public.revoke_agent_key(current_setting('test.id_a')::uuid)$$,
  '42501',
  'permission denied for function revoke_agent_key',
  'authenticated cannot revoke keys'
);
reset role;
select public.revoke_agent_key(current_setting('test.id_a')::uuid);

set local role service_role;
select is(
  public.agent_key_user(encode(extensions.digest(current_setting('test.key_a'), 'sha256'), 'hex')),
  null::uuid,
  'revoked key resolves to null'
);
reset role;

select * from finish();
rollback;
