import type { Device } from "./device-list-store";

const STALE_MS = 24 * 3600_000;

/**
 * Ids of devices that look like an earlier install of a phone listed again: same name and
 * platform as a device seen later, and silent for more than 24 h. Only a hint; nothing is
 * merged, because a second phone may share the name.
 */
export function oldInstallIds(devices: Device[], now = Date.now()): Set<string> {
  const seen = (d: Device) => Date.parse(d.last_seen_at);
  return new Set(
    devices
      .filter(
        (d) =>
          now - seen(d) > STALE_MS &&
          devices.some((o) => o.id !== d.id && o.name === d.name && o.platform === d.platform && seen(o) > seen(d)),
      )
      .map((d) => d.id),
  );
}
