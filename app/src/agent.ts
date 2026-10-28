/** The agent that asks for the location. `name` is what the user sees. */
export type Agent = { clientId: string; name: string };

/** Ids shared by many agents: "always allow" would cover strangers, so it is never offered. */
const GENERIC = new Set(["unknown", "agent-key"]);

export const isGeneric = (clientId: string) => GENERIC.has(clientId);
