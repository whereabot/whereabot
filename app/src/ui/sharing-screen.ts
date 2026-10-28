import { $ } from "../status-display-dom";
import { emailProblem } from "../sharing/share-email-validation";
import { listShares, myEmail, shareWith, stopSharing, type Share, type Shares } from "../sharing/location-shares-store";
import { confirmStopSharing } from "./stop-sharing-confirm-dialog";
import { mountSharingSection } from "./sharing-section-markup";
import { sharedByMeRow, sharedWithMeRow } from "./sharing-row";
import { announce, setScreen } from "./screen-view-helpers";

export type SharingView = { kind: "loading" } | { kind: "error"; message: string } | ({ kind: "ready" } & Shares);

const say = (text: string) => ($("sharing-status").textContent = text);
const why = (e: unknown) => (e instanceof Error ? e.message : String(e));

/** Draws one state of the page. Data comes in as arguments so screenshots can seed it. */
export function renderSharing(view: SharingView, onStop: (s: Share) => void = () => {}) {
  const ready = view.kind === "ready";
  const byMe = ready ? view.byMe : [];
  const withMe = ready ? view.withMe : [];
  $("shared-by-me").replaceChildren(...byMe.map((s) => sharedByMeRow(s, onStop)));
  $("shared-with-me").replaceChildren(...withMe.map(sharedWithMeRow));
  $("by-me-empty").hidden = !ready || byMe.length > 0;
  $("with-me-empty").hidden = !ready || withMe.length > 0;
  $("sharing-retry").hidden = view.kind !== "error";
  say(view.kind === "loading" ? "Loading…" : view.kind === "error" ? `Could not load sharing: ${view.message}` : "");
}

async function refresh(quiet = false) {
  if (!quiet) renderSharing({ kind: "loading" });
  try {
    renderSharing({ kind: "ready", ...(await listShares()) }, onStopSharing);
  } catch (e) {
    renderSharing({ kind: "error", message: why(e) });
  }
}

export async function onStopSharing(s: Share) {
  if (!(await confirmStopSharing(s.grantee_email))) return;
  try {
    await stopSharing(s.id);
  } catch (e) {
    return say(`Could not stop sharing: ${why(e)}`);
  }
  await refresh(true);
  say(`Stopped sharing with ${s.grantee_email}.`);
  $("sharing-title").focus();
}

async function onSubmit(e: Event) {
  e.preventDefault();
  const input = $<HTMLInputElement>("share-email");
  const problem = emailProblem(input.value, await myEmail().catch(() => ""));
  input.setAttribute("aria-invalid", problem ? "true" : "false");
  $("share-error").textContent = problem;
  if (problem) return input.focus();
  const button = $<HTMLButtonElement>("share-submit");
  button.disabled = true;
  try {
    const email = input.value.trim().toLowerCase();
    const done = (await shareWith(email)) === "ok" ? `Sharing with ${email}.` : `You already share with ${email}.`;
    input.value = "";
    await refresh(true);
    say(done);
    announce(done);
  } catch (err) {
    $("share-error").textContent = `Could not share: ${why(err)}`;
    input.focus();
  } finally {
    button.disabled = false;
  }
}

export function initSharing() {
  mountSharingSection();
  $("open-sharing").addEventListener("click", () => {
    void refresh();
    setScreen("sharing");
    $("sharing-title").focus();
  });
  $("sharing-back").addEventListener("click", () => {
    setScreen("home");
    $("open-sharing").focus();
  });
  $("sharing-retry").addEventListener("click", () => void refresh());
  $("share-form").addEventListener("submit", (e) => void onSubmit(e));
}
