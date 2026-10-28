import WebSocket from "@tauri-apps/plugin-websocket";
import { checkPermissions, getCurrentPosition, requestPermissions } from "@tauri-apps/plugin-geolocation";
import { answer, type Ping } from "./protocol";
import { $, setStatus } from "./dom";

const RETRY_MS = 5000;

let ws: WebSocket | null = null;
let retryTimer: number | undefined;
let stopped = true;

/** Returns an error message, or null when the URL may be used. */
export function checkUrl(server: string): string | null {
  let url: URL;
  try {
    url = new URL(server);
  } catch {
    return "Relay URL is not a valid URL";
  }
  const local = /^(localhost|10\.\d+\.\d+\.\d+|192\.168\.\d+\.\d+)$/.test(url.hostname);
  if (url.protocol === "wss:" || (url.protocol === "ws:" && local)) return null;
  return "Relay URL must start with wss:// (ws:// only for localhost, 10.x or 192.168.x)";
}

async function ensurePermission() {
  let perms = await checkPermissions();
  if (perms.location === "prompt" || perms.location === "prompt-with-rationale") {
    perms = await requestPermissions(["location"]);
  }
  if (perms.location !== "granted") throw new Error("Location permission denied");
}

function retry(server: string, token: string) {
  retryTimer = window.setTimeout(() => connect(server, token), RETRY_MS);
}

async function connect(server: string, token: string) {
  stopped = false;
  clearTimeout(retryTimer);
  setStatus("Connecting…");
  let socket: WebSocket;
  try {
    socket = await WebSocket.connect(`${server.replace(/\/$/, "")}/v1/device`, {
      headers: { Authorization: `Bearer ${token}` },
    });
  } catch (e) {
    setStatus(`Connect failed: ${e}. Retrying…`);
    retry(server, token);
    return;
  }
  if (stopped) {
    await socket.disconnect();
    return;
  }
  ws = socket;
  setStatus("Connected. Waiting for pings.");
  socket.addListener(async (msg) => {
    if (msg.type === "Close") {
      ws = null;
      if (stopped) return;
      setStatus("Disconnected. Retrying…");
      retry(server, token);
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
}

export async function start(server: string, token: string) {
  const err = checkUrl(server);
  if (err) return setStatus(err);
  try {
    await ensurePermission();
  } catch (e) {
    setStatus(String(e));
    return;
  }
  await connect(server, token);
}
