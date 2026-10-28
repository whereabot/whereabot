import type WebSocket from "@tauri-apps/plugin-websocket";

let live: WebSocket | null = null;
let deadline = 0; // latest ping expires_ms: how long an answer may wait for a live socket
let waiters: Array<() => void> = [];

export const getLive = () => live;

export function setLive(socket: WebSocket | null) {
  live = socket;
  if (socket) waiters.splice(0).forEach((w) => w());
}

/** Forgets the live socket and closes it. */
export function dropLive() {
  const old = live;
  live = null;
  void old?.disconnect().catch(() => {});
}

/** Remembers the ping's expiry so a queued answer stops waiting when the relay does. */
export function noteExpiry(data: string) {
  deadline = Number(/"expires_ms":\s*(\d+)/.exec(data)?.[1]) || deadline;
}

/** Sends on the CURRENT live socket; waits for a reconnect until the ping expires. */
export async function send(message: string): Promise<void> {
  const until = deadline > Date.now() ? deadline : Date.now() + 5000;
  for (;;) {
    const s = live;
    if (s) {
      try {
        return await s.send(message);
      } catch (e) {
        if (live === s) live = null;
        if (Date.now() >= until) throw e;
      }
    }
    const left = until - Date.now();
    if (left <= 0) throw new Error("phone not connected");
    await new Promise<void>((resolve) => {
      waiters.push(resolve);
      window.setTimeout(resolve, left);
    });
  }
}
