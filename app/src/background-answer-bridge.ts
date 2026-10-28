import { invoke } from "@tauri-apps/api/core";
import { relayUrl } from "./build-config";

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

/** Client ids the user chose "Always allow" for on a notification; cleared once taken. */
export const takeAlwaysAllowed = () =>
  call<{ clientIds: string[] }>("take_always_allowed").then((r) => r.clientIds, () => [] as string[]);

export type AutoShared = { clientId: string; name: string; at: number };

/** Answers the native handler sent on its own (silent push), and whether a "checked" tap opened the app. */
export const takeAutoShared = () =>
  call<{ shared: AutoShared[]; openHistory: boolean }>("take_auto_shared").catch(() => ({ shared: [], openHistory: false }));

/** Call once at start: gives the native handler the relay and Supabase addresses. */
export function initBackgroundAnswer() {
  if (!backgroundAnswerSupported) return;
  void call("configure", {
    relayUrl: relayUrl.replace(/^ws/, "http"),
    supabaseUrl: import.meta.env.VITE_SUPABASE_URL,
    anonKey: import.meta.env.VITE_SUPABASE_ANON_KEY,
  }).catch(() => {});
}
