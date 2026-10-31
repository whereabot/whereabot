import type { Device } from "./device-list-store";

const STALE_MS = 24 * 3600_000;

/**
 * Ids of rows that look like an earlier install of THIS device: same platform and same install
 * way (App, PWA, Browser) as the current row, seen before it and silent for more than 24 h.
 * Never the current row. A Browser row and a PWA row are different installs, so they never
 * mark each other. Only a hint; nothing is merged, because a second phone may look the same.
 */
export function oldInstallIds(devices: Device[], thisId: string, now = Date.now()): Set<string> {
  const me = devices.find((d) => d.id === thisId);
  if (!me?.app_kind) return new Set();
  const seen = (d: Device) => Date.parse(d.last_seen_at);
  return new Set(
    devices
      .filter(
        (d) =>
          d.id !== me.id &&
          d.platform === me.platform &&
          d.app_kind === me.app_kind &&
          seen(d) < seen(me) &&
          now - seen(d) > STALE_MS,
      )
      .map((d) => d.id),
  );
}
