import { checkPermissions, clearWatch, watchPosition, type Position } from "@tauri-apps/plugin-geolocation";

export type Fix = { pos: Position } | { fail: "permission" | "no_fix" };

/** Good enough to stop early: a street-level GPS fix. */
const PRECISE_M = 30;
/** Longest wait; the best reading so far answers then. */
const MAX_WAIT_MS = 10_000;

/**
 * One fix for one approved request. iOS often reports a coarse cell/Wi-Fi reading first,
 * so watch for up to 10 s and keep the most accurate reading instead of the first one.
 */
export async function getFix(): Promise<Fix> {
  try {
    const perms = await checkPermissions();
    if (perms.location !== "granted") return { fail: "permission" };
  } catch {
    return { fail: "permission" };
  }
  let best: Position | null = null;
  let watchId: number | null = null;
  const pos = await new Promise<Position | null>((resolve) => {
    const timer = window.setTimeout(() => resolve(best), MAX_WAIT_MS);
    watchPosition({ enableHighAccuracy: true, timeout: MAX_WAIT_MS, maximumAge: 0 }, (p) => {
      if (!p) return;
      if (!best || p.coords.accuracy < best.coords.accuracy) best = p;
      if (p.coords.accuracy <= PRECISE_M) {
        window.clearTimeout(timer);
        resolve(p);
      }
    })
      .then((id) => (watchId = id))
      .catch(() => {
        window.clearTimeout(timer);
        resolve(best);
      });
  });
  if (watchId !== null) void clearWatch(watchId).catch(() => {});
  return pos ? { pos } : { fail: "no_fix" };
}
