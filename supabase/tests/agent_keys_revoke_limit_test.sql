-- Limits and revocation: 10 active keys per user, and revoked keys stop resolving.
begin;
create extension if not exists pgtap with schema extensions;
select plan(4);

insert into auth.users (id, email) values
  ('00000000-0000-0000-0000-00000000000a', 'a@test.local');

set local role authenticated;
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000a","role":"authenticated"}', true);
select set_config('test.key_a', public.create_agent_key('laptop'), true);
select set_config('test.id_a', (select id::text from public.agent_keys), true);

select lives_ok(
  $$select count(public.create_agent_key('bulk-' || g)) from generate_series(2, 10) g$$,
  'user A creates keys 2 to 10'
);
select throws_ok(
  $$select public.create_agent_key('eleventh')$$,
  'P0001',
  'agent key limit reached (max 10 active keys)',
  'the 11th active key raises'
);
select lives_ok(
  $$select public.revoke_agent_key(current_setting('test.id_a')::uuid)$$,
  'user A revokes own key'
);
reset role;

set local role service_role;
select is(
  public.agent_key_user(encode(extensions.digest(current_setting('test.key_a'), 'sha256'), 'hex')),
  null::uuid,
  'revoked key resolves to null'
);
reset role;

select * from finish();
rollback;
