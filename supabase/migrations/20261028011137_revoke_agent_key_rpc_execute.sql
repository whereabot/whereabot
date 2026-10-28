-- Agent keys leave the app. Users no longer create or revoke keys through RPCs.
-- The table and agent_key_user stay for the headless fallback (WB_AGENT_KEYS=1).
revoke execute on function public.create_agent_key(text) from authenticated;
revoke execute on function public.revoke_agent_key(uuid) from authenticated;
