import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
import { inTauri } from "./tauri-runtime";

// The PWA uses the web Notification API (absent in iOS Safari until the page is on the Home Screen).
const web = () => ("Notification" in window ? Notification : null);

export async function notificationsGranted(): Promise<boolean> {
  if (!inTauri) return web()?.permission === "granted";
  return isPermissionGranted();
}

/** Asks once, up front (onboarding). Never called mid-request. */
export async function requestNotificationPermission(): Promise<string> {
  try {
    if (await notificationsGranted()) return "granted";
    if (!inTauri) return (await web()?.requestPermission()) ?? "denied";
    return await requestPermission();
  } catch {
    return "denied";
  }
}

/** Local notice so the user sees the prompt when the app is not in focus. Best effort. */
export async function notify(body: string, title = "Whereabot") {
  try {
    if (!(await notificationsGranted())) return;
    if (inTauri) sendNotification({ title, body });
    else new Notification(title, { body });
  } catch {
    // The in-app prompt still shows.
  }
}
