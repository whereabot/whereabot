import { $, setState, setStatus, showView } from "./status-display-dom";
import { disconnect, start } from "./relay-connection";
import { listenForAuthRedirect, signInWithGoogle, signInWithPassword, signOut } from "./supabase-sign-in";
import { devLogin } from "./build-config";
import { initDeviceIdentity } from "./devices/device-identity";
import { initDevices } from "./ui/devices-screen";
import { initHistory } from "./ui/history-screen";
import { initHome } from "./ui/home-screen";
import { initPermissions } from "./ui/permissions-screen";
import { forgetPush, initPush } from "./push/push-token-registration";
import { initBackgroundAnswer } from "./background-answer-bridge";
import { clearLocalData } from "./request-history-store";
import { getAccessToken, supabase } from "./supabase-client";

/** Read on every (re)connect. No session means the sign-in screen shows. */
async function currentToken() {
  await initDeviceIdentity(); // the hello that follows needs the Keychain id
  const token = await getAccessToken();
  if (!token) showView("signed-out");
  return token;
}

window.addEventListener("DOMContentLoaded", () => {
  void initDeviceIdentity(); // before the first hello; the sign-in round trip is far slower
  $("dev-form").hidden = !devLogin;
  $("google-sign-in").addEventListener("click", () => void signInWithGoogle());
  $("dev-form").addEventListener("submit", (e) => {
    e.preventDefault();
    void signInWithPassword(
      $<HTMLInputElement>("dev-email").value,
      $<HTMLInputElement>("dev-password").value,
    );
  });
  $("connect").addEventListener("click", () => void start(currentToken, true));
  $("disconnect").addEventListener("click", () => disconnect());
  $("sign-out").addEventListener("click", async () => {
    disconnect(true);
    await forgetPush(); // needs the session, so before signOut
    clearLocalData();
    await signOut();
  });
  initHistory();
  initDevices();
  initHome();
  initPermissions();
  initPush();
  initBackgroundAnswer();
  void listenForAuthRedirect();

  supabase.auth.onAuthStateChange((event, session) => {
    // Deferred: supabase calls must not run inside the auth callback.
    setTimeout(() => {
      if (!session) {
        disconnect(true);
        showView("signed-out");
        return;
      }
      showView("signed-in");
      // TOKEN_REFRESHED and friends must not undo a Disconnect.
      if (event === "SIGNED_IN" || event === "INITIAL_SESSION") start(currentToken);
    }, 0);
  });
  setStatus("Signed out.");
  setState("disconnected");
});

// Screenshot builds only: lets scripts/screens drive the UI without a relay.
if (import.meta.env.VITE_SCREENS) {
  void Promise.all([import("./ui/request-screen"), import("./ui/history-screen"), import("./ui/screen-view-helpers"), import("./ui/devices-screen")]).then(([r, h, v, d]) => {
    Object.assign(window, { __ui: { ...r, ...h, ...v, ...d } });
  });
}
