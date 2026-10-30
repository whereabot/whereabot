import { isTauri } from "@tauri-apps/api/core";

/** False in the browser build (PWA at /app): plugins are absent there, so callers use web APIs. */
export const inTauri = isTauri();

const ua = navigator.userAgent;
/**
 * Tauri on a phone. The geolocation plugin is a stub on desktop (always "prompt", zeroed fix),
 * so desktop Tauri uses the webview's navigator.geolocation, like the browser build.
 * iPadOS reports a Mac user agent; touch points tell them apart.
 */
export const mobileShell = inTauri && (/iPhone|iPad|iPod|Android/.test(ua) || (/Macintosh/.test(ua) && navigator.maxTouchPoints > 1));
