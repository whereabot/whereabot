import { openUrl } from "@tauri-apps/plugin-opener";
import { $ } from "../status-display-dom";
import { COPY, askPermission, permState, type Perm, type Step } from "./permission-step-content";
import { announce, setScreen } from "./screen-view-helpers";

let skipped = false;
let shown: Step | null = null;

function render(step: Step, state: Perm) {
  const c = COPY[step];
  const denied = state === "denied";
  const n = step === "location" ? "1 of 2 — Location" : "2 of 2 — Notifications";
  $("perm-step").textContent = "Step " + n;
  $("perm-title").textContent = c.title;
  $("perm-why").textContent = c.why;
  $("perm-status").textContent = denied ? "Not allowed. Turn it on in Settings." : "Not allowed yet";
  $("perm-ico-location").toggleAttribute("hidden", step !== "location");
  $("perm-ico-notifications").toggleAttribute("hidden", step !== "notifications");
  $("perm-how").hidden = !denied;
  $("perm-how").replaceChildren(...c.how.map((t) => Object.assign(document.createElement("li"), { textContent: t })));
  $("perm-allow").hidden = denied;
  $("perm-allow").textContent = c.button;
  $("perm-settings").hidden = !denied;
  $("perm-skip").hidden = step !== "notifications";
}

/** Runs on open and on every return to the foreground. */
async function refresh() {
  const screen = document.body.dataset.screen;
  if ($("signed-in").hidden || screen === "request" || screen === "history") return;
  const [loc, notif] = await Promise.all([permState("location"), permState("notifications")]);
  const step: Step | null = loc !== "granted" ? "location" : notif !== "granted" && !skipped ? "notifications" : null;
  $("notif-banner").hidden = step !== null || notif === "granted";
  if (!step) {
    if (document.body.dataset.screen === "permissions") setScreen("home");
    shown = null;
    return;
  }
  render(step, step === "location" ? loc : notif);
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
    skipped = false;
    void refresh();
  });
  $("perm-skip").addEventListener("click", () => {
    skipped = true;
    void refresh();
  });
  document.addEventListener("visibilitychange", () => document.visibilityState === "visible" && void refresh());
  // Sign-in reveals #signed-in; check then.
  new MutationObserver(() => void refresh()).observe($("signed-in"), { attributes: true, attributeFilter: ["hidden"] });
  void refresh();
}
