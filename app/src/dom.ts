export const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

export type ConnState = "connected" | "waiting" | "disconnected";

const LABELS: Record<ConnState, string> = {
  connected: "Connected",
  waiting: "Waiting",
  disconnected: "Disconnected",
};

export function setStatus(text: string) {
  $("status").textContent = text;
}

/** Colour and text both carry the state, so colour is never the only cue. */
export function setState(state: ConnState) {
  $("state-card").dataset.state = state;
  $("state-label").textContent = LABELS[state];
}

export function showView(view: "signed-in" | "signed-out") {
  $("signed-in").hidden = view !== "signed-in";
  $("signed-out").hidden = view !== "signed-out";
}
