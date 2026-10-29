import { isPermissionGranted } from "@tauri-apps/plugin-notification";
import { getToken, onTokenRefresh } from "tauri-plugin-mobile-push-api";
import { supabase } from "../supabase-client";
import { forgetToken, storeToken } from "./push-token-storage";
import { inTauri } from "../tauri-runtime";
import { forgetWebPush, onNotificationPermissionChange, registerWebPush } from "./web-push-subscription";

// Android would need FCM, which this app does not ship. The browser build uses Web Push instead.
const ios = inTauri && /iPhone|iPad|iPod/i.test(navigator.userAgent);
let refreshListening = false;

async function register() {
  try {
    const { data } = await supabase.auth.getSession();
    if (!data.session || !(await isPermissionGranted())) return;
    if (!refreshListening) {
      refreshListening = true;
      await onTokenRefresh(({ token }) => void storeToken(token));
    }
    await storeToken(await getToken());
  } catch (e) {
    console.warn("push registration failed", e);
  }
}

/**
 * Call once at start. Stores the APNs token (browser: the Web Push subscription) once the user
 * is signed in and has allowed notifications: on launch, on sign-in, and on every return to the foreground (the user may
 * have just allowed them). A tap on the push opens the app; the liveness code reconnects.
 */
export function initPush() {
  if (!ios && inTauri) return;
  const run = inTauri ? register : registerWebPush;
  if (!inTauri) onNotificationPermissionChange(() => void run());
  document.addEventListener("visibilitychange", () => document.visibilityState === "visible" && void run());
  supabase.auth.onAuthStateChange((event) => {
    if (event === "SIGNED_IN" || event === "INITIAL_SESSION") setTimeout(() => void run(), 0);
  });
}

/** Call in the sign-out handler, before `signOut()`. */
export const forgetPush = () => (inTauri ? forgetToken() : forgetWebPush());
