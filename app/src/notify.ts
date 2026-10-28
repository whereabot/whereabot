import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";

/** Local notice so the user sees the prompt when the app is not in focus. Best effort. */
export async function notify(body: string) {
  try {
    const granted = (await isPermissionGranted()) || (await requestPermission()) === "granted";
    if (granted) sendNotification({ title: "Whereabot", body });
  } catch {
    // The in-app prompt still shows.
  }
}
