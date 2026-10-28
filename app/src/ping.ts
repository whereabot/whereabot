import { isGeneric, type Agent } from "./agent";
import { answer, denied, unavailable, type Ping } from "./protocol";
import { $, setStatus } from "./dom";
import { resolveAgent } from "./agent-name";
import { addAlwaysApproved, isAlwaysApproved, recordRequest, type Outcome } from "./history";
import { notify } from "./notify";
import { getFix } from "./location";
import { showRequest } from "./ui/request";

// Used when an older relay sends no expires_ms.
const FALLBACK_MS = 55_000;
// Stop a little before the relay gives up, so our answer still lands.
const MARGIN_MS = 2_000;
// A reconnect makes the relay resend a ping: handle each id once.
const seen = new Set<string>();
// One prompt at a time: later pings wait here for their turn.
let queue: Promise<unknown> = Promise.resolve();

type Send = (message: string) => Promise<void>;

/** Answers one relay message if it is a locate ping. `send` targets the live socket. */
export async function answerPing(send: Send, data: string) {
  const ping = JSON.parse(data) as Ping;
  if (ping.type !== "locate" || seen.has(ping.id)) return;
  seen.add(ping.id);
  const agent = await resolveAgent(ping.client_id);
  if (isAlwaysApproved(agent.clientId)) return sendFix(send, ping, agent, "auto");
  const turn = queue.then(() => prompt(send, ping, agent));
  queue = turn.catch(() => {});
  return turn;
}

async function prompt(send: Send, ping: Ping, agent: Agent) {
  const left = (ping.expires_ms ? ping.expires_ms - Date.now() : FALLBACK_MS) - MARGIN_MS;
  if (left <= 0) return recordRequest(agent, "timeout"); // The relay already stopped waiting.
  const abort = new AbortController();
  const timer = window.setTimeout(() => abort.abort(), left);
  void notify(`Your agent ${agent.name} wants to know your location`);
  const choice = await showRequest(agent, abort.signal, !isGeneric(agent.clientId));
  clearTimeout(timer);
  if (choice === "reject") {
    recordRequest(agent, "rejected");
    return send(denied(ping, "user"));
  }
  if (choice === null) {
    recordRequest(agent, "timeout");
    return send(denied(ping, "no_answer"));
  }
  if (choice === "always") addAlwaysApproved(agent);
  await sendFix(send, ping, agent, choice === "always" ? "always" : "approved");
}

async function sendFix(send: Send, ping: Ping, agent: Agent, outcome: Outcome) {
  const fix = await getFix();
  if ("fail" in fix) {
    recordRequest(agent, fix.fail === "permission" ? "unavailable" : "failed");
    setStatus(fix.fail === "permission" ? "Location access is off. Turn it on in Settings." : "No location fix.");
    return send(unavailable(ping, fix.fail));
  }
  recordRequest(agent, outcome);
  await send(JSON.stringify(answer(ping, fix.pos)));
  $("last-ping").textContent = `Last ping answered at ${new Date().toLocaleTimeString()}`;
}
