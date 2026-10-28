import { $, setState, setStatus, showView } from "./dom";
import { disconnect, start } from "./connection";
import { listenForAuthRedirect, signInWithGoogle, signInWithPassword, signOut } from "./auth";
import { devLogin } from "./config";
import { initHistory } from "./ui/history";
import { initHome } from "./ui/home";
import { initPermissions } from "./ui/permissions";
import { clearLocalData } from "./history";
import { getAccessToken, supabase } from "./supabase";

/** Read on every (re)connect. No session means the sign-in screen shows. */
async function currentToken() {
  const token = await getAccessToken();
  if (!token) showView("signed-out");
  return token;
}

window.addEventListener("DOMContentLoaded", () => {
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
    clearLocalData();
    await signOut();
  });
  initHistory();
  initHome();
  initPermissions();
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
  void Promise.all([import("./ui/request"), import("./ui/history"), import("./ui/view")]).then(([r, h, v]) => {
    Object.assign(window, { __ui: { ...r, ...h, ...v } });
  });
}
