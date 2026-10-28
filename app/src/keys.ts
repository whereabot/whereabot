import { supabase } from "./supabase";
import { $, setStatus } from "./dom";

type AgentKey = { id: string; name: string; prefix: string };

export async function refreshKeys() {
  const { data, error } = await supabase
    .from("agent_keys")
    .select("id,name,prefix,created_at,last_used_at,revoked_at")
    .is("revoked_at", null)
    .order("created_at");
  if (error) return setStatus(`Could not load agent keys: ${error.message}`);
  renderKeys((data ?? []) as AgentKey[]);
}

function renderKeys(keys: AgentKey[]) {
  const items = keys.map((key) => {
    const li = document.createElement("li");
    const label = document.createElement("span");
    label.textContent = `${key.name} (${key.prefix}…) `;
    const revoke = document.createElement("button");
    revoke.type = "button";
    revoke.textContent = `Revoke ${key.name}`;
    revoke.addEventListener("click", () => void revokeKey(key.id));
    li.append(label, revoke);
    return li;
  });
  const empty = document.createElement("li");
  empty.textContent = "No agent keys yet.";
  $("key-list").replaceChildren(...(items.length ? items : [empty]));
}

async function revokeKey(id: string) {
  const { error } = await supabase.rpc("revoke_agent_key", { p_id: id });
  if (error) return setStatus(`Could not revoke key: ${error.message}`);
  setStatus("Key revoked.");
  await refreshKeys();
}

async function createKey(name: string) {
  const { data, error } = await supabase.rpc("create_agent_key", { p_name: name });
  if (error) return setStatus(`Could not create key: ${error.message}`);
  // The plaintext key is returned once. It is never fetched again.
  $<HTMLInputElement>("new-key").value = String(data);
  $("new-key-box").hidden = false;
  $<HTMLInputElement>("key-name").value = "";
  await refreshKeys();
}

export function initKeys() {
  $<HTMLFormElement>("key-form").addEventListener("submit", (e) => {
    e.preventDefault();
    void createKey($<HTMLInputElement>("key-name").value.trim());
  });
  $("copy-key").addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText($<HTMLInputElement>("new-key").value);
      setStatus("Key copied.");
    } catch (e) {
      setStatus(`Copy failed: ${e}`);
    }
  });
}
