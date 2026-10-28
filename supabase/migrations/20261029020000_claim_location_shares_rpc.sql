-- Called by the app on sign-in: binds shares addressed to the caller's VERIFIED email to the caller.
-- Security definer because grantees have no UPDATE right on location_shares.

create function public.claim_location_shares() returns integer
language plpgsql security definer set search_path = ''
as $$
declare claimed integer;
begin
  update public.location_shares s set grantee_user_id = auth.uid()
  from auth.users u
  where u.id = auth.uid() and u.email_confirmed_at is not null
    and s.grantee_email = lower(u.email)
    and s.revoked_at is null and s.grantee_user_id is distinct from auth.uid();
  get diagnostics claimed = row_count;
  return claimed;
end $$;

revoke all on function public.claim_location_shares() from public, anon;
grant execute on function public.claim_location_shares() to authenticated;
