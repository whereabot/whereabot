import { isTauri } from "@tauri-apps/api/core";

/** False in the browser build (PWA at /app): plugins are absent there, so callers use web APIs. */
export const inTauri = isTauri();
