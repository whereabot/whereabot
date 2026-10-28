import { checkPermissions, requestPermissions } from "@tauri-apps/plugin-geolocation";

export async function ensurePermission() {
  let perms = await checkPermissions();
  if (perms.location === "prompt" || perms.location === "prompt-with-rationale") {
    perms = await requestPermissions(["location"]);
  }
  if (perms.location !== "granted") throw new Error("Location permission denied");
}
