import { isTauri } from "@tauri-apps/api/core";
import { shellKind } from "./shell-kind";

/** False in the browser build (PWA at /app): plugins are absent there, so callers use web APIs. */
export const inTauri = isTauri();

const shell = shellKind(inTauri, navigator.userAgent, navigator.maxTouchPoints);
/** Tauri on a phone or iPad: the geolocation plugin. Elsewhere it is a stub (see shell-kind.ts). */
export const mobileShell = shell === "mobile";
/** Tauri on a Mac: CoreLocation commands (macos-location.ts). */
export const macShell = shell === "mac";
