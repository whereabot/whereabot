import { openUrl } from "@tauri-apps/plugin-opener";
import { getCurrent, onOpenUrl } from "@tauri-apps/plugin-deep-link";
import { supabase } from "./supabase";
import { setStatus } from "./dom";

const REDIRECT = "whereabouts://auth-callback";

export async function signInWithGoogle() {
  const { data, error } = await supabase.auth.signInWithOAuth({
    provider: "google",
    options: { redirectTo: REDIRECT, skipBrowserRedirect: true },
  });
  if (error) return setStatus(`Sign-in failed: ${error.message}`);
  await openUrl(data.url);
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
    const code = new URL(url).searchParams.get("code");
    if (!code) continue;
    const { error } = await supabase.auth.exchangeCodeForSession(code);
    if (error) setStatus(`Sign-in failed: ${error.message}`);
  }
}

/** Handles the redirect while the app runs, and when the redirect starts the app. */
export async function listenForAuthRedirect() {
  await onOpenUrl(completeRedirect);
  const initial = await getCurrent();
  if (initial) await completeRedirect(initial);
}
