import TauriWebSocket from "@tauri-apps/plugin-websocket";
import { connectBrowserSocket, type RelaySocket } from "./browser-relay-socket";
import { inTauri } from "./tauri-runtime";

export type { RelaySocket };

/** The app sends the token as a header; the browser build cannot, so it uses a subprotocol. */
export const openRelaySocket = (url: string, token: string): Promise<RelaySocket> =>
  inTauri
    ? TauriWebSocket.connect(url, { headers: { Authorization: `Bearer ${token}` } })
    : connectBrowserSocket(url, token);
