-- Coarse model hint ("390x844@3": short x long screen side @ pixel ratio), written on connect.
-- A reinstalled app or PWA has a new local id; it takes over the newest live row of the same
-- user, platform, app_kind and hint (app/src/devices/reinstall-device-match.ts), so two iPhones
-- of different sizes never share a row. Null = registered before the hint; still matchable.
alter table public.devices
  add column screen_hint text check (char_length(screen_hint) <= 32);
