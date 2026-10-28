-- always_allowed_agents with requester_user_id: owner manages requester rows for active grantees only.
begin;
create extension if not exists pgtap with schema extensions;
select plan(7);

insert into auth.users (id, email, email_confirmed_at) values
  ('00000000-0000-0000-0000-00000000000a', 'a@test.local', now()),
  ('00000000-0000-0000-0000-00000000000b', 'b@test.local', now()),
  ('00000000-0000-0000-0000-00000000000c', 'c@test.local', now());
insert into public.location_shares (owner_user_id, grantee_email, grantee_user_id, owner_display_name) values
  ('00000000-0000-0000-0000-00000000000a', 'b@test.local', '00000000-0000-0000-0000-00000000000b', 'Ay');

set local role authenticated;
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000a","role":"authenticated"}', true);
select lives_ok($$insert into public.always_allowed_agents (client_id) values ('own-agent')$$, 'own agent row (requester null) still works');
select lives_ok(
  $$insert into public.always_allowed_agents (client_id, requester_user_id) values ('own-agent', '00000000-0000-0000-0000-00000000000b')$$,
  'same client_id may be allowed for a requester too'
);
select throws_ok(
  $$insert into public.always_allowed_agents (client_id) values ('own-agent')$$,
  '23505', null, 'duplicate (user, client, null requester) is rejected'
);
select throws_ok(
  $$insert into public.always_allowed_agents (client_id, requester_user_id) values ('x', '00000000-0000-0000-0000-00000000000c')$$,
  '42501', null, 'requester row needs an active share'
);
select lives_ok($$update public.always_allowed_agents set client_name = 'N' where requester_user_id is not null$$, 'owner updates a requester row');

select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000a","role":"authenticated","client_id":"a-agent"}', true);
select throws_ok(
  $$insert into public.always_allowed_agents (client_id, requester_user_id) values ('y', '00000000-0000-0000-0000-00000000000b')$$,
  '42501', null, 'agent token cannot write requester rows'
);
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000b","role":"authenticated"}', true);
select is((select count(*)::int from public.always_allowed_agents), 0, 'the requester cannot read the owner rows directly');

select * from finish();
rollback;
