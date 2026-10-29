import { isPermissionGranted } from "@tauri-apps/plugin-notification";
import { getToken, onTokenRefresh } from "tauri-plugin-mobile-push-api";
import { supabase } from "../supabase-client";
import { forgetToken, storeToken } from "./push-token-storage";
import { inTauri } from "../tauri-runtime";

// Android would need FCM, which this app does not ship. The PWA has no Web Push (docs/install-from-browser.md).
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
 * Call once at start. Stores the APNs token once the user is signed in and has allowed
 * notifications: on launch, on sign-in, and on every return to the foreground (the user may
 * have just allowed them). A tap on the push opens the app; the liveness code reconnects.
 */
export function initPush() {
  if (!ios) return;
  document.addEventListener("visibilitychange", () => document.visibilityState === "visible" && void register());
  supabase.auth.onAuthStateChange((event) => {
    if (event === "SIGNED_IN" || event === "INITIAL_SESSION") setTimeout(() => void register(), 0);
  });
}

/** Call in the sign-out handler, before `signOut()`. */
export const forgetPush = forgetToken;
