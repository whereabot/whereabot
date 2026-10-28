import type { Agent } from "./agent";

export type Outcome = "approved" | "always" | "rejected" | "timeout" | "auto";
type Entry = { at: number; agent: Agent; outcome: Outcome };

const REQUESTS = "whereabouts.requests";
const ALWAYS = "whereabouts.alwaysApproved";
const CAP = 100;

function read<T>(key: string): T[] {
  try {
    return JSON.parse(localStorage.getItem(key) ?? "[]");
  } catch {
    return [];
  }
}

const write = (key: string, value: unknown) => localStorage.setItem(key, JSON.stringify(value));

/** Newest first. */
export const listRequests = (): Entry[] => read<Entry>(REQUESTS);

export function recordRequest(agent: Agent, outcome: Outcome) {
  write(REQUESTS, [{ at: Date.now(), agent, outcome }, ...listRequests()].slice(0, CAP));
}

export const listAlwaysApproved = (): Agent[] => read<Agent>(ALWAYS);

export const isAlwaysApproved = (clientId: string) =>
  listAlwaysApproved().some((a) => a.clientId === clientId);

export function addAlwaysApproved(agent: Agent) {
  write(ALWAYS, [...listAlwaysApproved().filter((a) => a.clientId !== agent.clientId), agent]);
}

export function removeAlwaysApproved(clientId: string) {
  write(ALWAYS, listAlwaysApproved().filter((a) => a.clientId !== clientId));
}
