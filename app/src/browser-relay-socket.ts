import type WebSocket from "@tauri-apps/plugin-websocket";
import type { Message } from "@tauri-apps/plugin-websocket";

/** The part of the Tauri socket the relay code uses. The browser socket below has the same shape. */
export type RelaySocket = Pick<WebSocket, "send" | "disconnect" | "addListener">;

/** Browsers cannot set Authorization on a WebSocket; the relay reads the token from this subprotocol pair. */
const BEARER_PROTOCOL = "whereabot.bearer";

/** Opens the relay socket from the browser build. Resolves once open, rejects if it closes first. */
export function connectBrowserSocket(url: string, token: string): Promise<RelaySocket> {
  const ws = new globalThis.WebSocket(url, [BEARER_PROTOCOL, token]);
  const listeners: Array<(m: Message) => void> = [];
  const emit = (m: Message) => listeners.forEach((l) => l(m));
  const socket: RelaySocket = {
    send: async (m) => {
      if (typeof m !== "string") throw new Error("the relay socket sends text only");
      ws.send(m);
    },
    disconnect: async () => ws.close(),
    addListener(cb) {
      listeners.push(cb);
      return () => void listeners.splice(listeners.indexOf(cb), 1);
    },
  };
  return new Promise((resolve, reject) => {
    ws.onopen = () => resolve(socket);
    // A refused upgrade (401 included) only shows as a close before open: no status reaches the page.
    ws.onclose = (e) => {
      reject(new Error(`socket closed (${e.code})`));
      emit({ type: "Close", data: { code: e.code, reason: e.reason } });
    };
    ws.onmessage = (e) => typeof e.data === "string" && emit({ type: "Text", data: e.data });
  });
}
