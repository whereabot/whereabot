export const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

export type ConnState = "connected" | "waiting" | "disconnected" | "replaced" | "expired";

const LABELS: Record<ConnState, string> = {
  connected: "Connected",
  waiting: "Waiting",
  disconnected: "Disconnected",
  replaced: "Replaced",
  expired: "Sign-in expired",
};

/** States that explain themselves in the status line. */
const NOTES: Partial<Record<ConnState, string>> = {
  replaced: "This device connected from somewhere else.",
  expired: "Sign-in expired. Sign out and sign in again.",
};

export function setStatus(text: string) {
  $("status").textContent = text;
}

/** Colour and text both carry the state, so colour is never the only cue. */
export function setState(state: ConnState) {
  $("state-card").dataset.state = state;
  $("state-label").textContent = LABELS[state];
  const note = NOTES[state];
  if (note) setStatus(note);
  // "replaced" allows a manual Connect. "expired" needs sign-out, so no button.
  $("connect").hidden = state !== "disconnected" && state !== "replaced";
  $("disconnect").hidden = state !== "connected" && state !== "waiting";
}

export function showView(view: "signed-in" | "signed-out") {
  document.body.dataset.screen = "home";
  $("signed-in").hidden = view !== "signed-in";
  $("signed-out").hidden = view !== "signed-out";
}
