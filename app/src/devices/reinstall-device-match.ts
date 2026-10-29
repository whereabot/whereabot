/** Which existing row a fresh install takes over. No imports, so `node --test` can load it as plain TS. */
export type OwnDeviceRow = {
  id: string;
  platform: string;
  app_kind: string | null;
  screen_hint: string | null;
  last_seen_at: string;
  revoked_at: string | null;
};

export type ThisInstall = { id: string; platform: string; appKind: string; screenHint: string };

/**
 * A row seen this recently may be a second phone of the same model that is in use now:
 * taking it over would share one id between two phones. The heartbeat writes every 60 s.
 * Trade-off: a reinstall done within 5 min of the old install's last beat makes a new row.
 */
export const LIVE_WINDOW_MS = 5 * 60_000;

/**
 * A reinstall (PWA re-added to the Home Screen, app reinstalled) loses the stored id.
 * Take over the newest live row with the same platform, install way and screen, so the
 * device list keeps one "App" and one "PWA" row per phone. Rows are the caller's own (RLS).
 * Null = keep this id: it already has a row (live or removed), it is a browser tab
 * (two browsers on one computer are two devices), the match may still be live, or nothing matches.
 */
export function rowToTakeOver(rows: OwnDeviceRow[], me: ThisInstall, nowMs: number): string | null {
  if (me.appKind === "browser" || rows.some((r) => r.id === me.id)) return null;
  const match = rows
    .filter((r) => !r.revoked_at && r.platform === me.platform && r.app_kind === me.appKind)
    // Null hint = registered before hints existed; still the same kind of install.
    .filter((r) => r.screen_hint === null || r.screen_hint === me.screenHint)
    .filter((r) => nowMs - Date.parse(r.last_seen_at) >= LIVE_WINDOW_MS)
    .sort((a, b) => b.last_seen_at.localeCompare(a.last_seen_at))[0];
  return match?.id ?? null;
}

/** Coarse model hint: two iPhones of different sizes stay apart. Orientation-proof. */
export function screenHint(width: number, height: number, ratio: number): string {
  return `${Math.min(width, height)}x${Math.max(width, height)}@${ratio}`;
}
