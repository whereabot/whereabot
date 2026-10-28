/**
 * The agent that asks for the location. `name` is what the user sees. `requesterId` is set when
 * the agent belongs to someone who shares with this user: `name` then already reads
 * "<Requester>'s agent <Agent>" and "Always allow" covers only that requester.
 */
export type Agent = { clientId: string; name: string; requesterId?: string };

/** Who asks, as the relay sends it with a shared request. */
export type Requester = { user_id: string; name?: string; email?: string };

/** Ids shared by many agents: "always allow" would cover strangers, so it is never offered. */
const GENERIC = new Set(["unknown", "agent-key"]);

export const isGeneric = (clientId: string) => GENERIC.has(clientId);

/** "Ofek's agent Claude" for a shared request; the plain agent otherwise. */
export function withRequester(agent: Agent, requester?: Requester | null): Agent {
  if (!requester?.user_id) return agent;
  const who = requester.name || requester.email || "Someone";
  return { ...agent, name: `${who}'s agent ${agent.name}`, requesterId: requester.user_id };
}

/** The same agent for the same requester (null requester = the user's own agent). */
export const sameAgent = (a: Agent, b: Agent) => a.clientId === b.clientId && (a.requesterId ?? null) === (b.requesterId ?? null);

/** The subject of the question: "Your agent X" or "Ofek's agent X". */
export const agentTitle = (a: Agent) => (a.requesterId ? a.name : `Your agent ${a.name}`);
