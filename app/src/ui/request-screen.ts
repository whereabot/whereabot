import { agentTitle, type Agent } from "../agent-identity";
import { $ } from "../status-display-dom";
import { announce, setScreen } from "./screen-view-helpers";

export type Choice = "approve" | "always" | "reject";

/**
 * Shows the request screen. Resolves with the user's choice, or null when aborted.
 * `canAlways` false hides "Always" (generic agent ids).
 */
export function showRequest(agent: Agent, signal: AbortSignal, canAlways = true): Promise<Choice | null> {
  if (signal.aborted) return Promise.resolve(null);
  return new Promise((resolve) => {
    const ui = new AbortController();
    const opts = { signal: ui.signal };
    const box = $("request");
    const allow = $("allow");
    const always = $("always-allow");
    const before = document.activeElement as HTMLElement | null;
    const finish = (choice: Choice | null) => {
      ui.abort();
      always.hidden = false;
    $("req-prefix").hidden = false;
      delete box.dataset.noAlways;
      setScreen("home");
      before?.focus?.();
      resolve(choice);
    };

    $("req-prefix").hidden = !!agent.requesterId; // a shared name already says whose agent it is
    $("req-name").textContent = agent.name;
    $("always-name").textContent = ` ${agent.name}`;
    always.hidden = !canAlways;
    if (canAlways) delete box.dataset.noAlways;
    else box.dataset.noAlways = "1";
    setScreen("request");
    announce(`${agentTitle(agent)} wants to know your location. Choose Reject, Allow or Always allow.`);
    $("request-title").focus();

    allow.addEventListener("click", () => finish("approve"), opts);
    $("reject").addEventListener("click", () => finish("reject"), opts);
    always.addEventListener("click", () => finish("always"), opts);
    signal.addEventListener("abort", () => finish(null), { once: true, signal: ui.signal });
  });
}
