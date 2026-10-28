import type { Agent } from "./agent";
import { supabase } from "./supabase";

/** Resolves the name the user saw when they authorized this OAuth client. */
export async function resolveAgent(clientId: string): Promise<Agent> {
  const { data } = await supabase.auth.oauth.listGrants();
  const name = data?.find((grant) => grant.client.id === clientId)?.client.name;
  // Agent keys and unknown clients have no grant: show a short id instead.
  return { clientId, name: name ?? clientId.slice(0, 8) };
}
