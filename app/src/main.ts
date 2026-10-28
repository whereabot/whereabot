import WebSocket from "@tauri-apps/plugin-websocket";
import {
  checkPermissions,
  getCurrentPosition,
  requestPermissions,
} from "@tauri-apps/plugin-geolocation";
import { answer, type Ping } from "./protocol";

const RETRY_MS = 5000;
const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

function setStatus(text: string) {
  $("status").textContent = text;
}

async function ensurePermission() {
  let perms = await checkPermissions();
  if (perms.location === "prompt" || perms.location === "prompt-with-rationale") {
    perms = await requestPermissions(["location"]);
  }
  if (perms.location !== "granted") throw new Error("Location permission denied");
}

async function connect(server: string, token: string) {
  setStatus("Connecting…");
  let ws: WebSocket;
  try {
    ws = await WebSocket.connect(`${server.replace(/\/$/, "")}/v1/device`, {
      headers: { Authorization: `Bearer ${token}` },
    });
  } catch (e) {
    setStatus(`Connect failed: ${e}. Retrying…`);
    setTimeout(() => connect(server, token), RETRY_MS);
    return;
  }
  setStatus("Connected. Waiting for pings.");
  ws.addListener(async (msg) => {
    if (msg.type === "Close") {
      setStatus("Disconnected. Retrying…");
      setTimeout(() => connect(server, token), RETRY_MS);
      return;
    }
    if (msg.type !== "Text") return;
    const ping = JSON.parse(msg.data) as Ping;
    if (ping.type !== "locate") return;
    const pos = await getCurrentPosition({ enableHighAccuracy: true, timeout: 20000, maximumAge: 0 });
    await ws.send(JSON.stringify(answer(ping, pos)));
    $("last-ping").textContent = `Last ping answered at ${new Date().toLocaleTimeString()}`;
  });
}

window.addEventListener("DOMContentLoaded", () => {
  const server = $<HTMLInputElement>("server");
  const token = $<HTMLInputElement>("token");
  server.value = localStorage.getItem("server") ?? "";
  // shortcut: token in localStorage, move to OS keychain (stronghold plugin) before 1.0.
  token.value = localStorage.getItem("token") ?? "";

  $("pair-form").addEventListener("submit", async (e) => {
    e.preventDefault();
    localStorage.setItem("server", server.value);
    localStorage.setItem("token", token.value);
    try {
      await ensurePermission();
    } catch (err) {
      setStatus(String(err));
      return;
    }
    await connect(server.value, token.value);
  });
});
