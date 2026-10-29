-- How an install runs: the native Tauri app, the PWA installed from /app, or a plain browser tab.
-- The client writes it on connect; null = not reported yet (shown as "Unknown").

alter table public.devices
  add column app_kind text check (app_kind in ('app', 'pwa', 'browser'));

-- Backfill what the push channel proves: APNs tokens come only from the native app,
-- and iOS gives Web Push only to home-screen installs. Other rows stay null until they reconnect.
update public.devices d set app_kind = 'app'
  where exists (select 1 from public.push_tokens t where t.device_id = d.id);
update public.devices d set app_kind = 'pwa'
  where app_kind is null and platform = 'ios'
    and exists (select 1 from public.web_push_subscriptions w where w.device_id = d.id);
