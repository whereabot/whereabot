import { $, setStatus, showView } from "./dom";
import { disconnect, start } from "./connection";
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
  void listenForAuthRedirect();

  supabase.auth.onAuthStateChange((_event, session) => {
    // Deferred: supabase calls must not run inside the auth callback.
    setTimeout(() => {
      if (!session) {
        disconnect();
        showView("signed-out");
        return;
      }
      showView("signed-in");
      void start(currentToken);
    }, 0);
  });
  setStatus("Signed out.");
});
