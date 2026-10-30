import { checkPermissions, requestPermissions } from "@tauri-apps/plugin-geolocation";
import { notificationsGranted, requestNotificationPermission } from "../local-notification";
import { browserLocationPermission, requestBrowserLocation } from "../browser-location-fix";
import { mobileShell } from "../tauri-runtime";
import { backgroundAnswerSupported, locationAuthorization, requestAlwaysLocation } from "../background-answer-bridge";

export type Step = "location" | "background" | "notifications";
export type Perm = "granted" | "prompt" | "denied";

const ASKED = "whereabouts.notif-asked";
const ASKED_ALWAYS = "whereabouts.always-asked";
const android = /Android/i.test(navigator.userAgent);

/** The steps this platform shows, in order. Only iOS answers from a notification. */
export const STEPS: Step[] = backgroundAnswerSupported ? ["location", "background", "notifications"] : ["location", "notifications"];

export const COPY = {
  location: {
    label: "Location",
    title: "Allow location",
    why: "So your agent can get your location. Only when you approve a request.",
    button: "Allow location",
    how: android
      ? ["Tap Open Settings.", "Tap Permissions, then Location, then choose Allow only while using the app.", "Come back to Whereabot."]
      : ["Tap Open Settings.", "Tap Location, then choose While Using the App.", "Come back to Whereabot."],
  },
  background: {
    label: "Answer from the notification",
    title: "Allow location when the app is closed",
    why: "So you can tap Allow on the notification without opening Whereabot. Whereabot still reads your location only when you approve a request. It never tracks you.",
    button: "Change to Always",
    how: ["Tap Open Settings.", "Tap Location, then choose Always.", "Come back to Whereabot."],
  },
  notifications: {
    label: "Notifications",
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
      const s = mobileShell ? (await checkPermissions()).location : await browserLocationPermission();
      return s === "granted" || s === "denied" ? s : "prompt";
    }
    if (step === "background") {
      if (!backgroundAnswerSupported) return "granted";
      const s = await locationAuthorization();
      if (s === "always") return "granted";
      // iOS shows the Always prompt once; after that only Settings can change it.
      return s === "denied" || localStorage.getItem(ASKED_ALWAYS) ? "denied" : "prompt";
    }
    if (await notificationsGranted()) return "granted";
  } catch {
    return "denied";
  }
  return localStorage.getItem(ASKED) ? "denied" : "prompt";
}

export async function askPermission(step: Step) {
  try {
    if (step === "location") await (mobileShell ? requestPermissions(["location"]) : requestBrowserLocation());
    else if (step === "background") {
      localStorage.setItem(ASKED_ALWAYS, "1");
      await requestAlwaysLocation();
    } else {
      localStorage.setItem(ASKED, "1");
      await requestNotificationPermission();
    }
  } catch {
    // The re-check shows the result.
  }
}
