import { type Agent, isGeneric } from "./agent";
import { supabase } from "./supabase";

export type Outcome =
  | "approved" | "always" | "rejected" | "timeout" | "auto" | "failed" | "unavailable";
type Entry = { at: number; agent: Agent; outcome: Outcome };

const REQUESTS = "whereabouts.requests";
const ALWAYS = "whereabouts.alwaysApproved";
const UID = "whereabouts.uid";
const CAP = 100;

// Storage is per user, so a second account on this phone never sees the first one's data.
let uid = localStorage.getItem(UID) ?? "anon";
supabase.auth.onAuthStateChange((_e, session) => {
  if (!session) return;
  uid = session.user.id;
  localStorage.setItem(UID, uid);
});

const key = (base: string) => `${base}.${uid}`;

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

export function recordRequest(agent: Agent, outcome: Outcome) {
  write(REQUESTS, [{ at: Date.now(), agent, outcome }, ...listRequests()].slice(0, CAP));
}

export const listAlwaysApproved = (): Agent[] => read<Agent>(ALWAYS);

export const isAlwaysApproved = (clientId: string) =>
  !isGeneric(clientId) && listAlwaysApproved().some((a) => a.clientId === clientId);

export function addAlwaysApproved(agent: Agent) {
  if (isGeneric(agent.clientId)) return;
  write(ALWAYS, [...listAlwaysApproved().filter((a) => a.clientId !== agent.clientId), agent]);
}

export function removeAlwaysApproved(clientId: string) {
  write(ALWAYS, listAlwaysApproved().filter((a) => a.clientId !== clientId));
}

export const clearHistory = () => localStorage.removeItem(key(REQUESTS));

/** Sign-out: drops every user's history and always-allowed list from this phone. */
export function clearLocalData() {
  for (const k of Object.keys(localStorage)) {
    // First-run flag from ui/home.ts: the next account sees onboarding again.
    if (k === "whereabouts.onboarded") localStorage.removeItem(k);
    if ([REQUESTS, ALWAYS, UID].some((base) => k === base || k.startsWith(`${base}.`))) localStorage.removeItem(k);
  }
  uid = "anon";
}
