import { type Agent, isGeneric } from "./agent-identity";
import { supabase } from "./supabase-client";

// The relay reads this table to wake the phone silently instead of asking (docs/protocol.md).
const TABLE = "always_allowed_agents";

/** Upserts agents for the signed-in user. False on failure. */
export async function saveAlwaysAllowed(agents: Agent[]) {
  const rows = agents.filter((a) => !isGeneric(a.clientId)).map((a) => ({ client_id: a.clientId, client_name: a.name }));
  if (!rows.length) return true;
  const { error } = await supabase.from(TABLE).upsert(rows, { onConflict: "user_id,client_id" });
  if (error) console.warn("always-allowed save failed", error.message);
  return !error;
}

export async function deleteAlwaysAllowed(clientId: string) {
  const { error } = await supabase.from(TABLE).delete().eq("client_id", clientId);
  if (error) console.warn("always-allowed delete failed", error.message);
}

/** The user's rows, or null when the read fails (keep the local cache then). */
export async function fetchAlwaysAllowed(): Promise<Agent[] | null> {
  const { data, error } = await supabase.from(TABLE).select("client_id, client_name").order("created_at");
  if (error) return null;
  return data.map((r) => ({ clientId: r.client_id, name: r.client_name ?? "An unnamed agent" }));
}
