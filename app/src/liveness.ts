import { getLive } from "./outbox";

let lastSeen = 0;
/** Call on every frame from the relay. */
export const touch = () => (lastSeen = Date.now());

/** Calls `reconnect` on network return, on return from background, and after 60 s of silence. */
export function watchLiveness(reconnect: () => void) {
  window.addEventListener("online", reconnect);
  // iOS kills the socket in the background without a Close event; short hides (prompts) keep it.
  let hiddenAt = 0;
  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "hidden") return void (hiddenAt = Date.now());
    if (Date.now() - hiddenAt >= 5000) reconnect();
  });
  // The relay pings every 25 s; silence means a dead socket (e.g. after a deploy).
  window.setInterval(() => {
    if (getLive() && document.visibilityState === "visible" && Date.now() - lastSeen > 60_000) reconnect();
  }, 10_000);
}
