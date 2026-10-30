-- An agent's OAuth token is a full user JWT, so it reaches PostgREST too. The relay only reads and
-- prunes devices, push_tokens and web_push_subscriptions with it. Inserts and updates now need the
-- user's own app session (no `client_id` claim), as on always_allowed_agents: an agent cannot
-- un-revoke a device or register its own push endpoint. Deletes stay open for the relay's pruning.

drop policy devices_insert_own on public.devices;
drop policy devices_update_own on public.devices;
create policy devices_insert_own on public.devices
  for insert to authenticated
  with check (user_id = (select auth.uid()) and (select auth.jwt() ->> 'client_id') is null);
create policy devices_update_own on public.devices
  for update to authenticated
  using (user_id = (select auth.uid()) and (select auth.jwt() ->> 'client_id') is null)
  with check (user_id = (select auth.uid()) and (select auth.jwt() ->> 'client_id') is null);

drop policy push_tokens_insert_own on public.push_tokens;
drop policy push_tokens_update_own on public.push_tokens;
create policy push_tokens_insert_own on public.push_tokens
  for insert to authenticated
  with check (user_id = (select auth.uid()) and (select auth.jwt() ->> 'client_id') is null);
create policy push_tokens_update_own on public.push_tokens
  for update to authenticated
  using (user_id = (select auth.uid()) and (select auth.jwt() ->> 'client_id') is null)
  with check (user_id = (select auth.uid()) and (select auth.jwt() ->> 'client_id') is null);

drop policy web_push_subscriptions_insert_own on public.web_push_subscriptions;
drop policy web_push_subscriptions_update_own on public.web_push_subscriptions;
create policy web_push_subscriptions_insert_own on public.web_push_subscriptions
  for insert to authenticated
  with check (user_id = (select auth.uid()) and (select auth.jwt() ->> 'client_id') is null);
create policy web_push_subscriptions_update_own on public.web_push_subscriptions
  for update to authenticated
  using (user_id = (select auth.uid()) and (select auth.jwt() ->> 'client_id') is null)
  with check (user_id = (select auth.uid()) and (select auth.jwt() ->> 'client_id') is null);
