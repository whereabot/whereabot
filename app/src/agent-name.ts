import { type Agent, isGeneric } from "./agent";
import { supabase } from "./supabase";

const LOOKUP_CAP_MS = 3000;
let names = new Map<string, string>();
let loading: Promise<void> | null = null;

/** Loads the OAuth grants into the cache. Call on connect; a failure keeps the old cache. */
export function refreshGrants(): Promise<void> {
  loading = supabase.auth.oauth
    .listGrants()
    .then((r) => {
      if (r.data) names = new Map(r.data.map((g) => [g.client.id, g.client.name]));
    })
    .catch(() => {})
    .finally(() => (loading = null));
  return loading;
}

supabase.auth.onAuthStateChange((event) => {
  if (event === "SIGNED_IN" || event === "INITIAL_SESSION") setTimeout(() => void refreshGrants(), 0);
  if (event === "SIGNED_OUT") names = new Map();
});

/** The name the user saw when they authorized this client. A fallback is shown, never cached. */
export async function resolveAgent(id: string | undefined): Promise<Agent> {
  // Older relays send no client_id; a failed lookup must never block the prompt.
  const clientId = id ?? "unknown";
  if (!names.has(clientId) && !isGeneric(clientId)) {
    const cap = new Promise<void>((done) => setTimeout(done, LOOKUP_CAP_MS));
    await Promise.race([loading ?? refreshGrants(), cap]);
  }
  return { clientId, name: names.get(clientId) ?? (isGeneric(clientId) ? "An agent" : "An unnamed agent") };
}
