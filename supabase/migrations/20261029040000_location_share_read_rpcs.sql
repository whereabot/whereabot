-- Share-checked reads for the relay, which runs with the requester's JWT. Each refuses (empty/false)
-- unless an ACTIVE share exists with the caller as grantee (id or verified email).
-- security definer + empty search_path.

create function public.share_target_push_tokens(p_owner uuid)
returns setof text
language sql stable security definer set search_path = ''
as $$
  select t.token
  from public.push_tokens t left join public.devices d on d.id = t.device_id
  where t.user_id = p_owner and t.platform = 'ios' and (d.id is null or d.revoked_at is null)
    and exists (select 1 from public.location_shares s
      where s.owner_user_id = p_owner and s.revoked_at is null
        and (s.grantee_user_id = auth.uid() or s.grantee_email = lower(auth.jwt() ->> 'email')))
$$;

create function public.is_always_allowed_for_requester(p_owner uuid, p_client_id text)
returns boolean
language sql stable security definer set search_path = ''
as $$
  select exists (
    select 1 from public.always_allowed_agents a
    where a.user_id = p_owner and a.client_id = p_client_id and a.requester_user_id = auth.uid()
      and exists (select 1 from public.location_shares s
        where s.owner_user_id = p_owner and s.revoked_at is null
          and (s.grantee_user_id = auth.uid() or s.grantee_email = lower(auth.jwt() ->> 'email'))))
$$;

revoke all on function public.share_target_push_tokens(uuid),
  public.is_always_allowed_for_requester(uuid, text) from public, anon;
grant execute on function public.share_target_push_tokens(uuid),
  public.is_always_allowed_for_requester(uuid, text) to authenticated;
