import type { Agent } from "../agent";
import { $ } from "../dom";
import { enableSlide } from "./slide";
import { announce, setScreen } from "./view";

export type Choice = "approve" | "always" | "reject";

/**
 * Shows the request screen. Resolves with the user's choice, or null when aborted.
 * `canAlways` false hides "always allow" (generic agent ids) and turns the slide off.
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
      box.dataset.slide = "idle";
      always.hidden = false;
      delete box.dataset.noAlways;
      setScreen("home");
      before?.focus?.();
      resolve(choice);
    };

    for (const id of ["req-name", "req-name-always", "always-name"]) $(id).textContent = agent.name;
    box.dataset.slide = "idle";
    always.hidden = !canAlways;
    if (canAlways) delete box.dataset.noAlways;
    else box.dataset.noAlways = "1";
    setScreen("request");
    announce(`Your agent ${agent.name} wants to know your location. Choose Allow or Reject.`);
    $("request-title").focus();

    allow.addEventListener("click", () => allow.dataset.dragged || finish("approve"), opts);
    $("reject").addEventListener("click", () => finish("reject"), opts);
    always.addEventListener("click", () => finish("always"), opts);
    if (canAlways) {
      enableSlide(allow, $("reject"), (s) => (box.dataset.slide = s), () => finish("always"), ui.signal);
    }
    signal.addEventListener("abort", () => finish(null), { once: true, signal: ui.signal });
  });
}
