import type { Agent } from "../agent";
import { $ } from "../dom";
import { enableSlide } from "./slide";
import { announce, setScreen } from "./view";

export type Choice = "approve" | "always" | "reject";

/** Shows the request screen. Resolves with the user's choice, or null when aborted. */
export function showRequest(agent: Agent, signal: AbortSignal): Promise<Choice | null> {
  if (signal.aborted) return Promise.resolve(null);
  return new Promise((resolve) => {
    const ui = new AbortController();
    const opts = { signal: ui.signal };
    const box = $("request");
    const allow = $("allow");
    const finish = (choice: Choice | null) => {
      ui.abort();
      box.dataset.slide = "idle";
      setScreen("home");
      resolve(choice);
    };

    for (const id of ["req-name", "req-name-always", "always-name"]) $(id).textContent = agent.name;
    box.dataset.slide = "idle";
    setScreen("request");
    announce(`Your agent ${agent.name} wants to know your location. Choose Allow or Reject.`);
    $("request-title").focus();

    allow.addEventListener("click", () => allow.dataset.dragged || finish("approve"), opts);
    $("reject").addEventListener("click", () => finish("reject"), opts);
    $("always-allow").addEventListener("click", () => finish("always"), opts);
    enableSlide(allow, $("reject"), (s) => (box.dataset.slide = s), () => finish("always"), ui.signal);
    signal.addEventListener("abort", () => finish(null), { once: true, signal: ui.signal });
  });
}
