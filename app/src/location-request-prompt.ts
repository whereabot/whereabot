import { isGeneric, type Agent } from "./agent-identity";
import { answer, denied, log, unavailable, type Ping } from "./relay-wire-protocol";
import { $, setStatus } from "./status-display-dom";
import { resolveAgent } from "./agent-display-name";
import { addAlwaysApproved, isAlwaysApproved, recordRequest, type Outcome } from "./request-history-store";
import { notify } from "./local-notification";
import { getFix } from "./device-location-fix";
import { showRequest } from "./ui/request-screen";
import { cancelRequest, track, untrack, wasCancelled } from "./devices/cancelled-request-tracking";

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
  const ping = JSON.parse(data) as Ping | { type: "cancel"; id: string };
  if (ping.type === "cancel") return cancelRequest(ping.id); // another device answered
  if (ping.type !== "locate" || seen.has(ping.id)) return;
  seen.add(ping.id);
  const step = (m: string) => void send(log(`${ping.id.slice(0, 8)} ${m}`)).catch(() => {});
  step("ping received");
  const agent = await resolveAgent(ping.client_id);
  step(`agent resolved, always=${isAlwaysApproved(agent.clientId)}`);
  if (isAlwaysApproved(agent.clientId)) return sendFix(send, ping, agent, "auto");
  const turn = queue.then(() => prompt(send, ping, agent, step));
  queue = turn.catch(() => {});
  return turn;
}

async function prompt(send: Send, ping: Ping, agent: Agent, step: (m: string) => void) {
  if (wasCancelled(ping.id)) return recordRequest(agent, "elsewhere");
  const left = (ping.expires_ms ? ping.expires_ms - Date.now() : FALLBACK_MS) - MARGIN_MS;
  if (left <= 0) return recordRequest(agent, "timeout"); // The relay already stopped waiting.
  const abort = new AbortController();
  const timer = window.setTimeout(() => abort.abort(), left);
  track(ping.id, abort);
  void notify(`Your agent ${agent.name} wants to know your location`);
  const shown = showRequest(agent, abort.signal, !isGeneric(agent.clientId));
  step(`prompt shown, left=${left}ms, screen=${document.body.dataset.screen}, visible=${document.visibilityState}`);
  const choice = await shown;
  clearTimeout(timer);
  untrack(ping.id);
  if (wasCancelled(ping.id)) return recordRequest(agent, "elsewhere");
  step(`choice=${choice}`);
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
