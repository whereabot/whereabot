-- RLS: the user's app session manages own always-allowed agents; an agent token
-- (JWT with client_id) can read them but never add, change or remove one.
begin;
create extension if not exists pgtap with schema extensions;
select plan(10);

insert into auth.users (id, email) values
  ('00000000-0000-0000-0000-00000000000a', 'a@test.local'),
  ('00000000-0000-0000-0000-00000000000b', 'b@test.local');
insert into public.always_allowed_agents (user_id, client_id, client_name) values
  ('00000000-0000-0000-0000-00000000000b', 'agent-of-b', 'B Bot');

set local role authenticated;
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000a","role":"authenticated"}', true);

select lives_ok(
  $$insert into public.always_allowed_agents (client_id, client_name) values ('claude-code', 'Claude Code')$$,
  'A always-allows an agent (user_id defaults to auth.uid())'
);
select is((select count(*)::int from public.always_allowed_agents), 1, 'A sees only own rows');
select throws_ok(
  $$insert into public.always_allowed_agents (user_id, client_id) values ('00000000-0000-0000-0000-00000000000b', 'evil')$$,
  '42501', null, 'A cannot add a row for B'
);
select throws_ok(
  $$insert into public.always_allowed_agents (client_id) values ('agent-key')$$,
  '23514', null, 'generic client ids are rejected'
);
with d as (delete from public.always_allowed_agents where client_id = 'agent-of-b' returning 1) select is(count(*)::int, 0, 'A cannot delete B row') from d;

-- The same user, through an agent's OAuth token.
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000a","role":"authenticated","client_id":"sneaky-agent"}', true);
select is((select count(*)::int from public.always_allowed_agents where client_id = 'claude-code'), 1, 'agent token reads its user rows');
select throws_ok(
  $$insert into public.always_allowed_agents (client_id) values ('sneaky-agent')$$,
  '42501', null, 'agent token cannot always-allow itself'
);
with d as (delete from public.always_allowed_agents where client_id = 'claude-code' returning 1) select is(count(*)::int, 0, 'agent token cannot delete a row') from d;

select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000a","role":"authenticated"}', true);
with d as (delete from public.always_allowed_agents where client_id = 'claude-code' returning 1) select is(count(*)::int, 1, 'A removes own row') from d;

set local role anon;
select throws_ok($$select count(*) from public.always_allowed_agents$$, '42501', null, 'anon has no access');

select * from finish();
rollback;
