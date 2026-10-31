import { openUrl } from "@tauri-apps/plugin-opener";
import { getCurrent, onOpenUrl } from "@tauri-apps/plugin-deep-link";
import { supabase } from "./supabase-client";
import { $, setStatus } from "./status-display-dom";
import { inTauri } from "./tauri-runtime";

// The PWA comes back to its own page; Supabase allows https://<site>/app/ (infra supabase.tf).
const REDIRECT = inTauri ? "whereabouts://auth-callback" : new URL(import.meta.env.BASE_URL, location.origin).href;

/** One browser tab at a time: the button stays off until the user is back. */
function setBusy(busy: boolean) {
  for (const id of ["apple-sign-in", "google-sign-in"]) $<HTMLButtonElement>(id).disabled = busy;
}
document.addEventListener("visibilitychange", () => document.visibilityState === "visible" && setBusy(false));

/** Same browser round trip for both: Apple goes through the Services ID (infra apple-sign-in-variables.tf). */
export async function signInWith(provider: "apple" | "google") {
  setBusy(true);
  const { data, error } = await supabase.auth.signInWithOAuth({
    provider,
    options: { redirectTo: REDIRECT, skipBrowserRedirect: inTauri },
  });
  if (error) {
    setBusy(false);
    return setStatus(`Sign-in failed: ${error.message}`);
  }
  if (inTauri) await openUrl(data.url); // the browser build navigates by itself
}

/** Dev only: local Supabase testing with email and password. */
export async function signInWithPassword(email: string, password: string) {
  const { error } = await supabase.auth.signInWithPassword({ email, password });
  if (error) setStatus(`Sign-in failed: ${error.message}`);
}

export async function signOut() {
  const { error } = await supabase.auth.signOut();
  if (error) setStatus(`Sign-out failed: ${error.message}`);
}

async function completeRedirect(urls: string[]) {
  for (const url of urls) {
    if (!url.startsWith(REDIRECT)) continue;
    const u = new URL(url);
    const hash = new URLSearchParams(u.hash.slice(1));
    const get = (k: string) => u.searchParams.get(k) ?? hash.get(k);
    const err = get("error");
    if (err) {
      setBusy(false);
      setStatus(err === "access_denied" ? "Sign-in cancelled" : `Sign-in failed: ${get("error_description") ?? err}`);
      continue;
    }
    const code = u.searchParams.get("code");
    if (!code) continue;
    const { error } = await supabase.auth.exchangeCodeForSession(code);
    if (error) setStatus(`Sign-in failed: ${error.message}`);
  }
}

/** Handles the redirect while the app runs, and when the redirect starts the app. */
export async function listenForAuthRedirect() {
  if (!inTauri) {
    if (!location.search && !location.hash) return;
    const url = location.href;
    history.replaceState(null, "", location.pathname); // the code is single-use: keep it out of history
    return completeRedirect([url]);
  }
  await onOpenUrl(completeRedirect);
  const initial = await getCurrent();
  if (initial) await completeRedirect(initial);
}
