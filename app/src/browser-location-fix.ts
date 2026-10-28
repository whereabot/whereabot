import type { Fix } from "./device-location-fix";

/** Same rule as the native path: stop at a 30 m fix, else answer with the best reading after 10 s. */
const PRECISE_M = 30;
const MAX_WAIT_MS = 10_000;

/** Browser build: one fix from navigator.geolocation. The page must be open (no background answer). */
export function getBrowserFix(): Promise<Fix> {
  if (!("geolocation" in navigator)) return Promise.resolve({ fail: "no_fix" });
  return new Promise<Fix>((resolve) => {
    let best: GeolocationPosition | null = null;
    const finish = (fix: Fix) => {
      navigator.geolocation.clearWatch(id);
      window.clearTimeout(timer);
      resolve(fix);
    };
    const timer = window.setTimeout(() => finish(best ? { pos: best } : { fail: "no_fix" }), MAX_WAIT_MS);
    const id = navigator.geolocation.watchPosition(
      (p) => {
        if (!best || p.coords.accuracy < best.coords.accuracy) best = p;
        if (p.coords.accuracy <= PRECISE_M) finish({ pos: p });
      },
      (e) => {
        if (e.code === e.PERMISSION_DENIED) finish({ fail: "permission" });
        // Other errors: keep waiting; the timer answers with the best reading so far.
      },
      { enableHighAccuracy: true, timeout: MAX_WAIT_MS, maximumAge: 0 },
    );
  });
}

/** "granted", "denied" or "prompt". Never reads the location itself. */
export async function browserLocationPermission(): Promise<"granted" | "denied" | "prompt"> {
  // Safari before 16 has no Permissions API: report "prompt" so the Allow button shows.
  const s = await navigator.permissions?.query({ name: "geolocation" }).catch(() => null);
  return s?.state ?? "prompt";
}

/** Shows the browser's location prompt by asking for one coarse reading. */
export const requestBrowserLocation = () =>
  new Promise<void>((resolve) => navigator.geolocation.getCurrentPosition(() => resolve(), () => resolve(), { maximumAge: Infinity }));
