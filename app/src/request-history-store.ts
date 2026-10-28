import { type Agent, isGeneric } from "./agent-identity";
import { supabase } from "./supabase-client";
import { deleteAlwaysAllowed, saveAlwaysAllowed } from "./always-allowed-server";

export type Outcome =
  | "approved" | "always" | "rejected" | "timeout" | "auto" | "failed" | "unavailable" | "elsewhere";
type Entry = { at: number; agent: Agent; outcome: Outcome };

const REQUESTS = "whereabouts.requests";
const ALWAYS = "whereabouts.alwaysApproved";
/** Set once this user's local list was copied to Supabase (always-allowed-sync.ts). */
export const ALWAYS_MIGRATED = "whereabouts.alwaysMigrated";
const UID = "whereabouts.uid";
const CAP = 100;

// Storage is per user, so a second account on this phone never sees the first one's data.
let uid = localStorage.getItem(UID) ?? "anon";
supabase.auth.onAuthStateChange((_e, session) => {
  if (!session) return;
  uid = session.user.id;
  localStorage.setItem(UID, uid);
});

/** Per-user localStorage key. */
export const key = (base: string) => `${base}.${uid}`;

function read<T>(base: string): T[] {
  try {
    return JSON.parse(localStorage.getItem(key(base)) ?? "[]");
  } catch {
    return [];
  }
}

const write = (base: string, value: unknown) => localStorage.setItem(key(base), JSON.stringify(value));

/** Newest first. */
export const listRequests = (): Entry[] => read<Entry>(REQUESTS);

export function recordRequest(agent: Agent, outcome: Outcome, at = Date.now()) {
  write(REQUESTS, [{ at, agent, outcome }, ...listRequests()].slice(0, CAP));
}

export const listAlwaysApproved = (): Agent[] => read<Agent>(ALWAYS);

export const isAlwaysApproved = (clientId: string) =>
  !isGeneric(clientId) && listAlwaysApproved().some((a) => a.clientId === clientId);

/** The local list is a cache of Supabase `always_allowed_agents`; writes go to both. */
export function addAlwaysApproved(agent: Agent) {
  if (isGeneric(agent.clientId)) return Promise.resolve(true);
  write(ALWAYS, [...listAlwaysApproved().filter((a) => a.clientId !== agent.clientId), agent]);
  return saveAlwaysAllowed([agent]);
}

export function removeAlwaysApproved(clientId: string) {
  write(ALWAYS, listAlwaysApproved().filter((a) => a.clientId !== clientId));
  void deleteAlwaysAllowed(clientId);
}

export const replaceAlwaysApproved = (agents: Agent[]) => write(ALWAYS, agents.filter((a) => !isGeneric(a.clientId)));

export const clearHistory = () => localStorage.removeItem(key(REQUESTS));

/** Sign-out: drops every user's history and always-allowed list from this phone. */
export function clearLocalData() {
  for (const k of Object.keys(localStorage)) {
    // First-run flag from ui/home-screen.ts: the next account sees onboarding again.
    if (k === "whereabouts.onboarded") localStorage.removeItem(k);
    if ([REQUESTS, ALWAYS, ALWAYS_MIGRATED, UID].some((base) => k === base || k.startsWith(`${base}.`))) localStorage.removeItem(k);
  }
  uid = "anon";
}
