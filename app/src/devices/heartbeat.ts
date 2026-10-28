import { syncDevice } from "./store";
import { signOutRemoved } from "./removed";

const EVERY_MS = 60_000;
let timer: number | undefined;

async function beat() {
  try {
    if ((await syncDevice()) === "revoked") await signOutRemoved();
  } catch (e) {
    // Offline or Supabase down: the next beat retries. Presence is best effort.
    console.warn("device heartbeat failed", e);
  }
}

/** Registers this device now, then refreshes last_seen_at every 60 s while connected. */
export function startHeartbeat() {
  stopHeartbeat();
  void beat();
  timer = window.setInterval(() => void beat(), EVERY_MS);
}

export function stopHeartbeat() {
  clearInterval(timer);
  timer = undefined;
}
