import { openUrl } from "@tauri-apps/plugin-opener";
import { mobileShell } from "../tauri-runtime";
import { $ } from "../status-display-dom";
import { COPY, STEPS, askPermission, permState, type Perm, type Step } from "./permission-step-content";
import { announce, setScreen } from "./screen-view-helpers";

// Location is required; the later steps can be skipped.
const skipped = new Set<Step>();
let shown: Step | null = null;

function render(step: Step, state: Perm) {
  const c = COPY[step];
  const denied = state === "denied";
  $("perm-step").textContent = `Step ${STEPS.indexOf(step) + 1} of ${STEPS.length} — ${c.label}`;
  $("perm-title").textContent = c.title;
  $("perm-why").textContent = c.why;
  $("perm-status").textContent = denied ? "Not allowed. Turn it on in Settings." : "Not allowed yet";
  $("perm-ico-location").toggleAttribute("hidden", step === "notifications");
  $("perm-ico-notifications").toggleAttribute("hidden", step !== "notifications");
  $("perm-how").hidden = !denied || !mobileShell;
  $("perm-how").replaceChildren(...c.how.map((t) => Object.assign(document.createElement("li"), { textContent: t })));
  $("perm-allow").hidden = denied;
  $("perm-allow").textContent = c.button;
  // A web page or desktop app cannot open phone Settings: the user changes it in the browser settings.
  $("perm-settings").hidden = !denied || !mobileShell;
  $("perm-skip").hidden = step === "location";
}

/** Runs on open and on every return to the foreground. */
async function refresh() {
  const screen = document.body.dataset.screen;
  if ($("signed-in").hidden || screen === "request" || screen === "history") return;
  const states = await Promise.all(STEPS.map(permState));
  const state = (s: Step) => states[STEPS.indexOf(s)];
  const step = STEPS.find((s) => state(s) !== "granted" && (s === "location" || !skipped.has(s))) ?? null;
  $("notif-banner").hidden = step !== null || state("notifications") === "granted";
  if (!step) {
    if (document.body.dataset.screen === "permissions") setScreen("home");
    shown = null;
    return;
  }
  render(step, state(step));
  const entering = document.body.dataset.screen !== "permissions" || shown !== step;
  shown = step;
  setScreen("permissions");
  if (entering) $("perm-title").focus();
}

export function initPermissions() {
  $("perm-allow").addEventListener("click", async () => {
    if (shown) await askPermission(shown);
    await refresh();
    announce($("perm-status").textContent ?? "");
  });
  $("perm-settings").addEventListener("click", () => void openUrl("app-settings:"));
  $("notif-fix").addEventListener("click", () => {
    skipped.delete("notifications");
    void refresh();
  });
  $("perm-skip").addEventListener("click", () => {
    if (shown) skipped.add(shown);
    void refresh();
  });
  document.addEventListener("visibilitychange", () => document.visibilityState === "visible" && void refresh());
  // Sign-in reveals #signed-in; check then.
  new MutationObserver(() => void refresh()).observe($("signed-in"), { attributes: true, attributeFilter: ["hidden"] });
  void refresh();
}
