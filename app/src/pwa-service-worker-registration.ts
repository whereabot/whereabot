import { inTauri } from "./tauri-runtime";

/** Browser build only: the worker makes the page installable (Add to Home Screen). */
export function registerServiceWorker() {
  if (inTauri || !("serviceWorker" in navigator)) return;
  void navigator.serviceWorker.register(`${import.meta.env.BASE_URL}sw.js`).catch((e) => console.warn("service worker", e));
}
