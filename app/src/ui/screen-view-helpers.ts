import { $ } from "../status-display-dom";

export type Screen = "home" | "history" | "request" | "permissions" | "devices";

/** CSS shows the section that matches body[data-screen]. */
export function setScreen(screen: Screen) {
  document.body.dataset.screen = screen;
}

/** Screen reader announcement, polite for history and assertive for requests. */
export function announce(text: string) {
  const live = $("announce");
  live.textContent = "";
  window.setTimeout(() => (live.textContent = text), 50);
}

export function el(tag: string, className: string, text = "") {
  const node = document.createElement(tag);
  node.className = className;
  node.textContent = text;
  return node;
}
