-- RPCs for agent keys. Each function is SECURITY DEFINER with an empty
-- search_path, so every object reference is schema-qualified.

create or replace function public.create_agent_key(p_name text)
returns text
language plpgsql
security definer
set search_path = ''
as $$
declare
  v_uid uuid := auth.uid();
  v_key text;
begin
  if v_uid is null then
    raise exception 'not authenticated';
  end if;

  -- Serialize per user so concurrent calls cannot both pass the cap.
  perform pg_catalog.pg_advisory_xact_lock(pg_catalog.hashtext(v_uid::text));

  if (select count(*) from public.agent_keys
      where user_id = v_uid and revoked_at is null) >= 10 then
    raise exception 'agent key limit reached (max 10 active keys)';
  end if;

  v_key := 'wb_' || encode(extensions.gen_random_bytes(24), 'hex');

  insert into public.agent_keys (user_id, name, prefix, key_hash)
  values (
    v_uid,
    p_name,
    left(v_key, 11),
    encode(extensions.digest(v_key, 'sha256'), 'hex')
  );

  return v_key;
end;
$$;

create or replace function public.revoke_agent_key(p_id uuid)
returns void
language sql
security definer
set search_path = ''
as $$
  update public.agent_keys
     set revoked_at = now()
   where id = p_id
     and user_id = auth.uid()
     and revoked_at is null;
$$;

create or replace function public.agent_key_user(p_key_hash text)
returns uuid
language plpgsql
security definer
set search_path = ''
as $$
declare
  v_user_id uuid;
begin
  update public.agent_keys
     set last_used_at = now()
   where key_hash = p_key_hash
     and revoked_at is null
  returning user_id into v_user_id;

  return v_user_id;
end;
$$;

revoke all on function public.create_agent_key(text) from public, anon, service_role;
grant execute on function public.create_agent_key(text) to authenticated;

revoke all on function public.revoke_agent_key(uuid) from public, anon, service_role;
grant execute on function public.revoke_agent_key(uuid) to authenticated;

revoke all on function public.agent_key_user(text) from public, anon, authenticated;
grant execute on function public.agent_key_user(text) to service_role;
