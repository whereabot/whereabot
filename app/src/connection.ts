import WebSocket from "@tauri-apps/plugin-websocket";
import { getCurrentPosition } from "@tauri-apps/plugin-geolocation";
import { answer, type Ping } from "./protocol";
import { $, setState, setStatus } from "./dom";
import { checkUrl, relayUrl } from "./config";
import { ensurePermission } from "./location";

const RETRY_MS = 5000;

type TokenSource = () => Promise<string | null>;

let ws: WebSocket | null = null;
let retryTimer: number | undefined;
let stopped = true;

function retry(getToken: TokenSource) {
  retryTimer = window.setTimeout(() => connect(getToken), RETRY_MS);
}

async function connect(getToken: TokenSource) {
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
      ws = null;
      if (stopped) return;
      setStatus("Connection lost. Retrying…");
      setState("waiting");
      retry(getToken);
      return;
    }
    if (msg.type !== "Text") return;
    const ping = JSON.parse(msg.data) as Ping;
    if (ping.type !== "locate") return;
    let pos;
    try {
      pos = await getCurrentPosition({ enableHighAccuracy: true, timeout: 20000, maximumAge: 0 });
    } catch (e) {
      setStatus(`Location failed: ${e}. No reply sent.`);
      return;
    }
    await socket.send(JSON.stringify(answer(ping, pos)));
    $("last-ping").textContent = `Last ping answered at ${new Date().toLocaleTimeString()}`;
  });
}

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
