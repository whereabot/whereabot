import WebSocket from "@tauri-apps/plugin-websocket";
import { answerPing } from "./ping";
import { setState, setStatus } from "./dom";
import { appVersion, checkUrl, relayUrl } from "./config";
import { backoffMs } from "./backoff";
import { dropLive, getLive, noteExpiry, send, setLive } from "./outbox";
import { touch, watchLiveness } from "./liveness";
import { authOk, EXPIRED, isAuthError, refreshOnce } from "./reauth";

export { send };
type TokenSource = () => Promise<string | null>;

let retryTimer: number | undefined;
let attempt = 0;
let connecting = false;
let tokenSource: TokenSource | null = null;
// True after Disconnect, "replaced" or "expired": only a manual start() clears it.
let halted = false;

function halt(state: "disconnected" | "replaced" | "expired", status: string) {
  halted = true;
  clearTimeout(retryTimer);
  dropLive();
  setStatus(status);
  setState(state);
}

function retry(delay: number) {
  clearTimeout(retryTimer);
  retryTimer = window.setTimeout(() => void connect(), delay);
}

async function connect() {
  if (halted || !tokenSource || connecting) return;
  connecting = true;
  await open(tokenSource).finally(() => (connecting = false));
}

async function open(getToken: TokenSource) {
  clearTimeout(retryTimer);
  setStatus("Connecting…");
  setState("waiting");
  const token = await getToken(); // supabase-js refreshes the session as needed
  if (!token) return halt("disconnected", "Signed out.");
  let socket: WebSocket;
  try {
    socket = await WebSocket.connect(`${relayUrl}/v1/device`, { headers: { Authorization: `Bearer ${token}` } });
  } catch (e) {
    if (isAuthError(e)) return (await refreshOnce()) ? retry(0) : halt("expired", EXPIRED);
    setStatus(`Connect failed: ${e}. Retrying…`);
    return retry(backoffMs(attempt++));
  }
  if (halted) return void socket.disconnect().catch(() => {});
  setLive(socket);
  attempt = 0;
  authOk();
  touch();
  setStatus("Connected. Waiting for pings.");
  setState("connected");
  void socket.send(JSON.stringify({ type: "hello", app_version: appVersion })).catch(() => {});
  socket.addListener((msg) => {
    if (getLive() !== socket) return;
    touch();
    if (msg.type === "Close") return onClose(msg.data?.code);
    if (msg.type !== "Text") return;
    noteExpiry(msg.data);
    answerPing(send, msg.data).catch((e) => setStatus(`Request failed: ${e}`));
  });
}

function onClose(code?: number) {
  setLive(null);
  if (halted) return;
  if (code === 4000) return halt("replaced", "Another phone is now connected for this account.");
  setStatus("Connection lost. Retrying…");
  setState("waiting");
  retry(code === 1012 ? 0 : backoffMs(attempt++));
}

watchLiveness(() => {
  if (halted) return;
  dropLive();
  attempt = 0;
  void connect();
});

/** `signOut` also lets the next sign-in connect on its own. */
export function disconnect(signOut = false) {
  halt("disconnected", "Disconnected.");
  halted = !signOut;
}

/** `manual` = the Connect button. Auth events pass false and never undo a halt. */
export function start(getToken: TokenSource, manual = false) {
  if (manual) halted = false;
  const err = checkUrl(relayUrl);
  if (err) return setStatus(err);
  tokenSource = getToken;
  if (!getLive()) void connect();
}
