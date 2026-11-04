import { agentTitle, type Agent } from "../agent-identity";
import { $ } from "../status-display-dom";
import { announce, setScreen } from "./screen-view-helpers";

export type Choice = "approve" | "always" | "reject";

/**
 * Shows the request screen. Resolves with the user's choice, or null when aborted.
 * `canAlways` false hides "Always" (generic agent ids). `reason` is the agent's own words, or null.
 */
export function showRequest(agent: Agent, signal: AbortSignal, canAlways = true, reason: string | null = null): Promise<Choice | null> {
  if (signal.aborted) return Promise.resolve(null);
  return new Promise((resolve) => {
    const ui = new AbortController();
    const opts = { signal: ui.signal };
    const box = $("request");
    const allow = $("allow");
    const always = $("always-allow");
    const before = document.activeElement as HTMLElement | null;
    const reasonEl = $("req-reason");
    // Cue marks text hidden below the cap; reaching the bottom clears it.
    const syncMore = () => {
      const cut = reasonEl.scrollHeight > reasonEl.clientHeight + 1 && reasonEl.scrollTop + reasonEl.clientHeight < reasonEl.scrollHeight - 1;
      if (cut) reasonEl.dataset.more = "1";
      else delete reasonEl.dataset.more;
    };
    const finish = (choice: Choice | null) => {
      ui.abort();
      always.hidden = false;
    $("req-prefix").hidden = false;
      reasonEl.hidden = true;
      delete reasonEl.dataset.more;
      delete box.dataset.noAlways;
      setScreen("home");
      before?.focus?.();
      resolve(choice);
    };

    $("req-prefix").hidden = !!agent.requesterId; // a shared name already says whose agent it is
    $("req-name").textContent = agent.name;
    $("req-reason-text").textContent = reason ?? "";
    reasonEl.hidden = !reason;
    $("always-name").textContent = ` ${agent.name}`;
    always.hidden = !canAlways;
    if (canAlways) delete box.dataset.noAlways;
    else box.dataset.noAlways = "1";
    setScreen("request");
    syncMore();
    reasonEl.addEventListener("scroll", syncMore, opts);
    announce(`${agentTitle(agent)} wants to know your location.${reason ? ` The agent says why: ${reason}.` : ""} Choose Reject, Allow or Always allow.`);
    $("request-title").focus();

    allow.addEventListener("click", () => finish("approve"), opts);
    $("reject").addEventListener("click", () => finish("reject"), opts);
    always.addEventListener("click", () => finish("always"), opts);
    signal.addEventListener("abort", () => finish(null), { once: true, signal: ui.signal });
  });
}
