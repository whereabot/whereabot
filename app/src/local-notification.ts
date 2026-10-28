import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";

/** Asks once, up front (onboarding). Never called mid-request. */
export async function requestNotificationPermission(): Promise<string> {
  try {
    if (await isPermissionGranted()) return "granted";
    return await requestPermission();
  } catch {
    return "denied";
  }
}

/** Local notice so the user sees the prompt when the app is not in focus. Best effort. */
export async function notify(body: string) {
  try {
    if (await isPermissionGranted()) sendNotification({ title: "Whereabot", body });
  } catch {
    // The in-app prompt still shows.
  }
}
