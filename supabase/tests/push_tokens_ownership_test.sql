-- Ownership: a user inserts, reads, updates and deletes only own push tokens.
begin;
create extension if not exists pgtap with schema extensions;
select plan(7);

insert into auth.users (id, email) values
  ('00000000-0000-0000-0000-00000000000a', 'a@test.local'),
  ('00000000-0000-0000-0000-00000000000b', 'b@test.local');
insert into public.push_tokens (user_id, token) values
  ('00000000-0000-0000-0000-00000000000b', 'token-b');

set local role authenticated;
select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000a","role":"authenticated"}', true);

insert into public.push_tokens (token) values ('token-a');
select is((select count(*)::int from public.push_tokens), 1, 'A sees only own token');
select throws_ok(
  $$insert into public.push_tokens (user_id, token) values ('00000000-0000-0000-0000-00000000000b', 'evil')$$,
  '42501', null, 'A cannot insert a row for B'
);
select lives_ok($$update public.push_tokens set updated_at = now() where token = 'token-a'$$, 'A updates own token');
select is((with d as (delete from public.push_tokens where token = 'token-b' returning 1) select count(*)::int from d), 0, 'A cannot delete B token');
select is((with d as (delete from public.push_tokens where token = 'token-a' returning 1) select count(*)::int from d), 1, 'A deletes own token');

select set_config('request.jwt.claims', '{"sub":"00000000-0000-0000-0000-00000000000b","role":"authenticated"}', true);
select is((select count(*)::int from public.push_tokens), 1, 'B still has own token');

set local role anon;
select throws_ok($$select count(*) from public.push_tokens$$, '42501', null, 'anon has no access');

select * from finish();
rollback;
