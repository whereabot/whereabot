import type WebSocket from "@tauri-apps/plugin-websocket";
import { getCurrentPosition } from "@tauri-apps/plugin-geolocation";
import { answer, type Ping } from "./protocol";
import { $, setStatus } from "./dom";

/** Answers one relay message on `socket` if it is a locate ping. */
export async function answerPing(socket: WebSocket, data: string) {
  const ping = JSON.parse(data) as Ping;
  if (ping.type !== "locate") return;
  let pos;
  try {
    // A fix up to 30 s old answers at once; a cold GPS fix can take longer.
    pos = await getCurrentPosition({ enableHighAccuracy: true, timeout: 15000, maximumAge: 30000 });
  } catch (e) {
    setStatus(`Location failed: ${e}. No reply sent.`);
    return;
  }
  await socket.send(JSON.stringify(answer(ping, pos)));
  $("last-ping").textContent = `Last ping answered at ${new Date().toLocaleTimeString()}`;
}
