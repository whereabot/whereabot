import { appVersion } from "../config";

export type Platform = "ios" | "android" | "desktop";

// Not under the per-user history keys, so sign-out keeps them: the install stays one device.
const ID = "whereabouts.device_id";
const NAME = "whereabouts.device_name";

/** randomUUID needs a secure context; getRandomValues works everywhere. */
function uuidV4(): string {
  const b = crypto.getRandomValues(new Uint8Array(16));
  b[6] = (b[6] & 0x0f) | 0x40;
  b[8] = (b[8] & 0x3f) | 0x80;
  const h = Array.from(b, (x) => x.toString(16).padStart(2, "0")).join("");
  return `${h.slice(0, 8)}-${h.slice(8, 12)}-${h.slice(12, 16)}-${h.slice(16, 20)}-${h.slice(20)}`;
}

/** Stable random id of this install. Created once. */
export function deviceId(): string {
  let id = localStorage.getItem(ID);
  if (!id) {
    id = uuidV4();
    localStorage.setItem(ID, id);
  }
  return id;
}

export function platform(): Platform {
  const ua = navigator.userAgent;
  if (/iPhone|iPad|iPod/.test(ua)) return "ios";
  return /Android/.test(ua) ? "android" : "desktop";
}

const DEFAULT_NAME: Record<Platform, string> = { ios: "iPhone", android: "Android phone", desktop: "Computer" };

export const deviceName = (): string => localStorage.getItem(NAME) ?? DEFAULT_NAME[platform()];

export const setDeviceName = (name: string) => localStorage.setItem(NAME, name);

/** After removal: the next sign-in on this install registers as a new device. */
export function resetIdentity() {
  localStorage.removeItem(ID);
  localStorage.removeItem(NAME);
}

/** First message on every socket. The relay keys the connection by `device_id`. */
export const helloMessage = () =>
  JSON.stringify({
    type: "hello",
    app_version: appVersion,
    device_id: deviceId(),
    device_name: deviceName(),
    platform: platform(),
  });
