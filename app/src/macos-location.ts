import { invoke } from "@tauri-apps/api/core";
import type { Position } from "@tauri-apps/plugin-geolocation";
import type { Fix } from "./device-location-fix";

// Mac app only: CoreLocation commands in app/src-tauri/src/macos_location.rs. They reject with "permission" or "no_fix".

/** One fix, same shape as the plugin's Position. */
export const getMacFix = (): Promise<Fix> =>
  invoke<Position>("macos_location").then(
    (pos): Fix => ({ pos }),
    (e): Fix => ({ fail: e === "permission" ? "permission" : "no_fix" }),
  );

/** "granted", "denied" or "prompt". With `request`, macOS shows its prompt once. */
export const macLocationPermission = (request: boolean) =>
  invoke<"granted" | "denied" | "prompt">("macos_location_permission", { request }).catch(() => "prompt" as const);
