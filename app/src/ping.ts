import { getCurrentPosition } from "@tauri-apps/plugin-geolocation";
import type { Agent } from "./agent";
import { answer, type Ping } from "./protocol";
import { $, setStatus } from "./dom";
import { resolveAgent } from "./agent-name";
import { addAlwaysApproved, isAlwaysApproved, recordRequest, type Outcome } from "./history";
import { notify } from "./notify";
import { ensurePermission } from "./location";
import { showRequest } from "./ui/request";

// The relay gives up on an unanswered ping after about a minute.
const PROMPT_MS = 55_000;
// A reconnect makes the relay resend a ping: handle each id once.
const seen = new Set<string>();

type Send = (message: string) => Promise<void>;

/** Answers one relay message if it is a locate ping. `send` targets the newest socket. */
export async function answerPing(send: Send, data: string) {
  const ping = JSON.parse(data) as Ping;
  if (ping.type !== "locate" || seen.has(ping.id)) return;
  seen.add(ping.id);
  const agent = await resolveAgent(ping.client_id);
  if (isAlwaysApproved(agent.clientId)) return sendFix(send, ping, agent, "auto");
  const abort = new AbortController();
  const timer = window.setTimeout(() => abort.abort(), PROMPT_MS);
  void notify(`Your agent ${agent.name} wants to know your location`);
  const choice = await showRequest(agent, abort.signal);
  clearTimeout(timer);
  if (choice === "reject") {
    recordRequest(agent, "rejected");
    return send(JSON.stringify({ type: "denied", id: ping.id }));
  }
  if (choice === null) return recordRequest(agent, "timeout");
  if (choice === "always") addAlwaysApproved(agent);
  await sendFix(send, ping, agent, choice === "always" ? "always" : "approved");
}

async function sendFix(send: Send, ping: Ping, agent: Agent, outcome: Outcome) {
  let pos;
  try {
    await ensurePermission();
    // A fix up to 30 s old answers at once; a cold GPS fix can take longer.
    pos = await getCurrentPosition({ enableHighAccuracy: true, timeout: 15000, maximumAge: 30000 });
  } catch (e) {
    setStatus(`Location failed: ${e}. No reply sent.`);
    return;
  }
  recordRequest(agent, outcome);
  await send(JSON.stringify(answer(ping, pos)));
  $("last-ping").textContent = `Last ping answered at ${new Date().toLocaleTimeString()}`;
}
