import { isPermissionGranted } from "@tauri-apps/plugin-notification";
import { checkPermissions, requestPermissions } from "@tauri-apps/plugin-geolocation";
import { requestNotificationPermission } from "../notify";

export type Step = "location" | "notifications";
export type Perm = "granted" | "prompt" | "denied";

const ASKED = "whereabouts.notif-asked";
const android = /Android/i.test(navigator.userAgent);

export const COPY = {
  location: {
    title: "Allow location",
    why: "So your agent can get your location. Only when you approve a request.",
    button: "Allow location",
    how: android
      ? ["Tap Open Settings.", "Tap Permissions, then Location, then choose Allow only while using the app.", "Come back to Whereabot."]
      : ["Tap Open Settings.", "Tap Location, then choose While Using the App.", "Come back to Whereabot."],
  },
  notifications: {
    title: "Allow notifications",
    why: "So you see when an agent asks.",
    button: "Allow notifications",
    how: android
      ? ["Tap Open Settings.", "Tap Permissions, then Notifications, then choose Allow.", "Come back to Whereabot."]
      : ["Tap Open Settings.", "Tap Notifications, then turn on Allow Notifications.", "Come back to Whereabot."],
  },
};

/** Checks only. Never reads the location itself. */
export async function permState(step: Step): Promise<Perm> {
  try {
    if (step === "location") {
      const s = (await checkPermissions()).location;
      return s === "granted" || s === "denied" ? s : "prompt";
    }
    if (await isPermissionGranted()) return "granted";
  } catch {
    return "denied";
  }
  return localStorage.getItem(ASKED) ? "denied" : "prompt";
}

export async function askPermission(step: Step) {
  try {
    if (step === "location") await requestPermissions(["location"]);
    else {
      localStorage.setItem(ASKED, "1");
      await requestNotificationPermission();
    }
  } catch {
    // The re-check shows the result.
  }
}
