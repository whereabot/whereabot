import {
  checkPermissions,
  getCurrentPosition,
  requestPermissions,
  type Position,
} from "@tauri-apps/plugin-geolocation";
import { requestNotificationPermission } from "./notify";

export type Fix = { pos: Position } | { fail: "permission" | "no_fix" };

const asking = (s: string) => s === "prompt" || s === "prompt-with-rationale";

/** Onboarding: ask for location and notification permission once, before any request. */
export async function requestPermissionsUpFront(): Promise<{ location: string; notifications: string }> {
  let location = "denied";
  try {
    let perms = await checkPermissions();
    if (asking(perms.location)) perms = await requestPermissions(["location"]);
    location = perms.location;
  } catch {
    // Reported as denied.
  }
  return { location, notifications: await requestNotificationPermission() };
}

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
