/** How this install runs. No imports, so `node --test` can load it as plain TS. */
export type AppKind = "app" | "pwa" | "browser";

/** Tauri shell = native app; installed to the home screen = PWA; otherwise a browser tab. */
export function detectAppKind(tauri: boolean, standalone: boolean): AppKind {
  return tauri ? "app" : standalone ? "pwa" : "browser";
}

const KIND_LABEL: Record<AppKind, string> = { app: "App", pwa: "Web app (PWA)", browser: "Browser" };

/** Badge text on the devices page. Null = registered before the kind was recorded. */
export const appKindLabel = (kind: AppKind | null): string => (kind ? KIND_LABEL[kind] : "Unknown");
