import {
  checkPermissions,
  getCurrentPosition,
  type Position,
} from "@tauri-apps/plugin-geolocation";

export type Fix = { pos: Position } | { fail: "permission" | "no_fix" };

/** One fix. Cold GPS can be slow, so a failed precise fix retries once with lower accuracy. */
export async function getFix(): Promise<Fix> {
  try {
    const perms = await checkPermissions();
    if (perms.location !== "granted") return { fail: "permission" };
  } catch {
    return { fail: "permission" };
  }
  try {
    // A fix up to 30 s old answers at once.
    return { pos: await getCurrentPosition({ enableHighAccuracy: true, timeout: 15000, maximumAge: 30000 }) };
  } catch {
    try {
      return { pos: await getCurrentPosition({ enableHighAccuracy: false, timeout: 10000, maximumAge: 120000 }) };
    } catch {
      return { fail: "no_fix" };
    }
  }
}
