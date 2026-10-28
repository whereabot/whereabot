import { $, setStatus, showView } from "./dom";
import { disconnect, start } from "./connection";
import { initKeys, refreshKeys } from "./keys";
import { listenForAuthRedirect, signInWithGoogle, signInWithPassword, signOut } from "./auth";
import { devLogin } from "./config";
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
  $("connect").addEventListener("click", () => void start(currentToken));
  $("disconnect").addEventListener("click", disconnect);
  $("sign-out").addEventListener("click", async () => {
    disconnect();
    await signOut();
  });
  initKeys();
  void listenForAuthRedirect();

  supabase.auth.onAuthStateChange((event, session) => {
    // Deferred: supabase calls must not run inside the auth callback.
    setTimeout(() => {
      if (!session) {
        disconnect();
        showView("signed-out");
        return;
      }
      showView("signed-in");
      if (event === "SIGNED_IN" || event === "INITIAL_SESSION") void refreshKeys();
      void start(currentToken);
    }, 0);
  });
  setStatus("Signed out.");
});
