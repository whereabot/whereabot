import type { Agent } from "./agent";
import { supabase } from "./supabase";

/** Resolves the name the user saw when they authorized this OAuth client. */
export async function resolveAgent(id: string | undefined): Promise<Agent> {
  // Older relays send no client_id; a failed lookup must never block the prompt.
  const clientId = id ?? "unknown";
  const { data } = await supabase.auth.oauth.listGrants().catch(() => ({ data: null }));
  const name = data?.find((grant) => grant.client.id === clientId)?.client.name;
  // Agent keys and unknown clients have no grant: show a short id instead.
  return { clientId, name: name ?? clientId.slice(0, 8) };
}
