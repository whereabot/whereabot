import { agentTitle, isGeneric, withRequester, type Agent } from "./agent-identity";
import { answer, denied, log, unavailable, type Ping } from "./relay-wire-protocol";
import { $, setStatus } from "./status-display-dom";
import { announce } from "./ui/screen-view-helpers";
import { resolveAgent } from "./agent-display-name";
import { addAlwaysApproved, isAlwaysApproved, recordRequest, type Outcome } from "./request-history-store";
import { notify } from "./local-notification";
import { getFix } from "./device-location-fix";
import { showRequest } from "./ui/request-screen";
import { showFirstAnswer } from "./ui/home-screen";
import { cancelRequest, track, untrack, wasCancelled } from "./devices/cancelled-request-tracking";
import { deliver } from "./relay-answer-http";

// Used when an older relay sends no expires_ms.
const FALLBACK_MS = 55_000;
// Stop a little before the relay gives up, so our answer still lands.
const MARGIN_MS = 2_000;
// A reconnect makes the relay resend a ping: handle each id once.
const seen = new Set<string>();
// One prompt at a time: later pings wait here for their turn.
let queue: Promise<unknown> = Promise.resolve();

type Send = (message: string) => Promise<void>;
type Step = (m: string) => void;

/** Answers go over acknowledged HTTP until the relay stops waiting. */
const deliverTo = (ping: Ping, body: string, step: Step) =>
  deliver(ping.id, body, ping.expires_ms ?? Date.now() + FALLBACK_MS, step);

/** Answers one relay message if it is a locate ping. `send` (the live socket) carries only step logs. */
export async function answerPing(send: Send, data: string) {
  const ping = JSON.parse(data) as Ping | { type: "cancel"; id: string };
  if (ping.type === "cancel") return cancelRequest(ping.id); // another device answered
  if (ping.type !== "locate" || seen.has(ping.id)) return;
  seen.add(ping.id);
  const step = (m: string) => void send(log(`${ping.id.slice(0, 8)} ${m}`)).catch(() => {});
  step("ping received");
  const agent = withRequester(await resolveAgent(ping.client_id), ping.requester);
  step(`agent resolved, always=${isAlwaysApproved(agent)}`);
  if (isAlwaysApproved(agent)) return sendFix(ping, agent, "auto", step);
  const turn = queue.then(() => prompt(ping, agent, step));
  queue = turn.catch(() => {});
  return turn;
}

async function prompt(ping: Ping, agent: Agent, step: Step) {
  if (wasCancelled(ping.id)) return recordRequest(agent, "elsewhere");
  const left = (ping.expires_ms ? ping.expires_ms - Date.now() : FALLBACK_MS) - MARGIN_MS;
  if (left <= 0) return recordRequest(agent, "timeout"); // The relay already stopped waiting.
  const abort = new AbortController();
  const timer = window.setTimeout(() => abort.abort(), left);
  track(ping.id, abort);
  void notify(`${agentTitle(agent)} wants to know your location`);
  const shown = showRequest(agent, abort.signal, !isGeneric(agent.clientId));
  step(`prompt shown, left=${left}ms, screen=${document.body.dataset.screen}, visible=${document.visibilityState}`);
  const choice = await shown;
  clearTimeout(timer);
  untrack(ping.id);
  if (wasCancelled(ping.id)) return recordRequest(agent, "elsewhere");
  step(`choice=${choice}`);
  if (choice === "reject") {
    recordRequest(agent, "rejected");
    return deliverTo(ping, denied(ping, "user"), step);
  }
  if (choice === null) {
    recordRequest(agent, "timeout");
    return deliverTo(ping, denied(ping, "no_answer"), step);
  }
  if (choice === "always") void addAlwaysApproved(agent);
  await sendFix(ping, agent, choice === "always" ? "always" : "approved", step);
}

async function sendFix(ping: Ping, agent: Agent, outcome: Outcome, step: Step) {
  const fix = await getFix();
  if ("fail" in fix) {
    recordRequest(agent, fix.fail === "permission" ? "unavailable" : "failed");
    setStatus(fix.fail === "permission" ? "Location access is off. Turn it on in Settings." : "No location fix.");
    return deliverTo(ping, unavailable(ping, fix.fail), step);
  }
  recordRequest(agent, outcome);
  await deliverTo(ping, JSON.stringify(answer(ping, fix.pos)), step);
  $("last-ping").textContent = `Last ping answered at ${new Date().toLocaleTimeString()}`;
  showFirstAnswer();
  if (outcome !== "auto") return;
  // Always allowed: nothing to decide, so say what happened (same words as the native handler).
  const checked = `${agent.name} checked your location`;
  setStatus(`${checked}.`);
  announce(checked);
  // Hidden (a background launch): the native handler owns the notice, so no second one.
  if (document.visibilityState === "visible") void notify("Shared just now. Manage in Whereabot.", checked);
}
