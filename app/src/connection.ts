import WebSocket from "@tauri-apps/plugin-websocket";
import { answerPing } from "./ping";
import { setState, setStatus } from "./dom";
import { checkUrl, relayUrl } from "./config";
import { ensurePermission } from "./location";

const RETRY_MS = 5000;

type TokenSource = () => Promise<string | null>;

let ws: WebSocket | null = null;
let retryTimer: number | undefined;
let stopped = true;

const retry = (getToken: TokenSource) => (retryTimer = window.setTimeout(() => connect(getToken), RETRY_MS));

let lastTokenSource: TokenSource | null = null;
async function connect(getToken: TokenSource) {
  lastTokenSource = getToken;
  clearTimeout(retryTimer);
  setStatus("Connecting…");
  setState("waiting");
  // Read the current session on every connect. supabase-js refreshes it.
  const token = await getToken();
  if (!token) {
    stopped = true;
    return;
  }
  let socket: WebSocket;
  try {
    socket = await WebSocket.connect(`${relayUrl}/v1/device`, {
      headers: { Authorization: `Bearer ${token}` },
    });
  } catch (e) {
    setStatus(`Connect failed: ${e}. Retrying…`);
    setState("waiting");
    retry(getToken);
    return;
  }
  if (stopped) {
    await socket.disconnect();
    return;
  }
  ws = socket;
  setStatus("Connected. Waiting for pings.");
  setState("connected");
  socket.addListener(async (msg) => {
    if (msg.type === "Close") {
      if (ws !== socket) return; // replaced by a newer socket
      ws = null;
      if (stopped) return;
      setStatus("Connection lost. Retrying…");
      setState("waiting");
      retry(getToken);
      return;
    }
    if (msg.type === "Text") await answerPing(socket, msg.data);
  });
}

// iOS kills the socket in the background, often without a Close event: reconnect on return.
// Short hides (location prompt, indicator) keep the socket and any ping in flight.
let hiddenAt = 0;
document.addEventListener("visibilitychange", () => {
  if (document.visibilityState === "hidden") return void (hiddenAt = Date.now());
  if (Date.now() - hiddenAt < 5000 || stopped || !lastTokenSource) return;
  const old = ws;
  ws = null;
  void old?.disconnect().catch(() => {});
  void connect(lastTokenSource);
});

export function disconnect() {
  stopped = true;
  clearTimeout(retryTimer);
  void ws?.disconnect();
  ws = null;
  setStatus("Disconnected.");
  setState("disconnected");
}

export async function start(getToken: TokenSource) {
  if (!stopped) return;
  const err = checkUrl(relayUrl);
  if (err) return setStatus(err);
  stopped = false;
  try {
    await ensurePermission();
  } catch (e) {
    stopped = true;
    setStatus(String(e));
    return;
  }
  await connect(getToken);
}
