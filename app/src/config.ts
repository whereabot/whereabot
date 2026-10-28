import pkg from "../package.json";

/** Build-time config from Vite env. See app/.env.example. */
export const appVersion: string = pkg.version;
export const relayUrl = import.meta.env.VITE_RELAY_URL.replace(/\/$/, "");
export const devLogin = import.meta.env.DEV || import.meta.env.VITE_DEV_LOGIN === "true";

/** Returns an error message, or null when the URL may be used. */
export function checkUrl(server: string): string | null {
  let url: URL;
  try {
    url = new URL(server);
  } catch {
    return "Relay URL is not a valid URL";
  }
  // 10.x covers the Android emulator host 10.0.2.2.
  const local = /^(localhost|10\.\d+\.\d+\.\d+|192\.168\.\d+\.\d+)$/.test(url.hostname);
  if (url.protocol === "wss:" || (url.protocol === "ws:" && local)) return null;
  return "Relay URL must start with wss:// (ws:// only for localhost, 10.x or 192.168.x)";
}
