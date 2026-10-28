import { invoke } from "@tauri-apps/api/core";
import { relayUrl } from "./build-config";
import { resolveAgent } from "./agent-display-name";
import { addAlwaysApproved } from "./request-history-store";
import { supabase } from "./supabase-client";

/** Only iOS has the native handler that answers from a notification action. */
export const backgroundAnswerSupported = /iPhone|iPad|iPod/i.test(navigator.userAgent);

const call = <T>(cmd: string, args?: Record<string, unknown>) =>
  invoke<T>(`plugin:background-answer|${cmd}`, args);

/** "always", "whenInUse", "denied" or "notDetermined". */
export const locationAuthorization = () =>
  call<{ status: string }>("location_authorization").then((r) => r.status);

/** Shows the "Change to Always Allow" prompt; resolves with the status after it closes. */
export const requestAlwaysLocation = () =>
  call<{ status: string }>("request_always_location").then((r) => r.status);

/** "Always allow" taps on a notification wait natively; they join this user's list here. */
async function mergeAlwaysAllowed() {
  const { data } = await supabase.auth.getSession();
  if (!data.session) return;
  const res = await call<{ clientIds: string[] }>("take_always_allowed").catch(() => null);
  for (const id of res?.clientIds ?? []) addAlwaysApproved(await resolveAgent(id));
}

/** Call once at start: gives the native handler the relay and Supabase addresses. */
export function initBackgroundAnswer() {
  if (!backgroundAnswerSupported) return;
  void call("configure", {
    relayUrl: relayUrl.replace(/^ws/, "http"),
    supabaseUrl: import.meta.env.VITE_SUPABASE_URL,
    anonKey: import.meta.env.VITE_SUPABASE_ANON_KEY,
  }).catch(() => {});
  document.addEventListener("visibilitychange", () => document.visibilityState === "visible" && void mergeAlwaysAllowed());
  supabase.auth.onAuthStateChange((event) => {
    // Deferred: the history store switches to this user's storage in its own listener first.
    if (event === "SIGNED_IN" || event === "INITIAL_SESSION") setTimeout(() => void mergeAlwaysAllowed(), 0);
  });
}
