import type { Agent } from "./agent-identity";
import { resolveAgent } from "./agent-display-name";
import { backgroundAnswerSupported, takeAlwaysAllowed } from "./background-answer-bridge";
import { fetchAlwaysAllowed, saveAlwaysAllowed } from "./always-allowed-server";
import { ALWAYS_MIGRATED, key, listAlwaysApproved, replaceAlwaysApproved } from "./request-history-store";
import { supabase } from "./supabase-client";

let running: Promise<void> | null = null;

const union = (a: Agent[], b: Agent[]) => [...a.filter((x) => !b.some((y) => y.clientId === x.clientId)), ...b];

/**
 * Supabase owns the always-allowed list (the relay reads it); localStorage is a cache.
 * Uploads "Always allow" taps the native handler kept, and once per user the list from
 * before the server knew. Then replaces the cache with the server rows.
 */
async function sync() {
  const { data } = await supabase.auth.getSession();
  if (!data.session) return;
  const tapped = backgroundAnswerSupported ? await takeAlwaysAllowed() : [];
  const fromNotifications = await Promise.all(tapped.map((id) => resolveAgent(id)));
  const migrated = localStorage.getItem(key(ALWAYS_MIGRATED)) === "1";
  if (!(await saveAlwaysAllowed([...(migrated ? [] : listAlwaysApproved()), ...fromNotifications]))) {
    // shortcut: offline keeps the whole local list for a re-upload, which can undo a removal made on another phone.
    replaceAlwaysApproved(union(listAlwaysApproved(), fromNotifications));
    localStorage.removeItem(key(ALWAYS_MIGRATED));
    return;
  }
  localStorage.setItem(key(ALWAYS_MIGRATED), "1");
  const rows = await fetchAlwaysAllowed();
  if (rows) replaceAlwaysApproved(rows);
}

const run = () => (running ??= sync().catch(() => {}).finally(() => (running = null)));

/** Call once at start: syncs on sign-in and on every return to the foreground. */
export function initAlwaysAllowedSync() {
  document.addEventListener("visibilitychange", () => document.visibilityState === "visible" && void run());
  supabase.auth.onAuthStateChange((event) => {
    // Deferred: the history store switches to this user's storage in its own listener first.
    if (event === "SIGNED_IN" || event === "INITIAL_SESSION") setTimeout(() => void run(), 0);
  });
}
