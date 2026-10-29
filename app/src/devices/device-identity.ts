import { invoke } from "@tauri-apps/api/core";
import { appVersion } from "../build-config";
import { inTauri } from "../tauri-runtime";
import { detectAppKind } from "./app-kind";

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

let current: string | null = null;
let started: Promise<string> | null = null;
const native = <T>(cmd: string, args?: Record<string, unknown>) => invoke<T>(`plugin:background-answer|${cmd}`, args);

/**
 * Call once at launch, before anything reads `deviceId()`. On iOS the id lives in the Keychain,
 * which survives uninstall (localStorage does not), so a reinstall stays one device. An id that
 * localStorage already holds moves into the Keychain. Elsewhere localStorage is the store.
 */
export const initDeviceIdentity = (): Promise<string> => (started ??= loadIdentity());

async function loadIdentity(): Promise<string> {
  const legacy = localStorage.getItem(ID);
  const res = await native<{ id: string }>("device_id", { legacy }).catch(() => null);
  current = res?.id ?? legacy ?? uuidV4();
  localStorage.setItem(ID, current);
  return current;
}

/** Stable random id of this install. Needs `initDeviceIdentity()` first; the fallback only covers a missed call. */
export function deviceId(): string {
  if (!current) {
    current = localStorage.getItem(ID) ?? uuidV4();
    localStorage.setItem(ID, current);
  }
  return current;
}

export function platform(): Platform {
  const ua = navigator.userAgent;
  if (/iPhone|iPad|iPod/.test(ua)) return "ios";
  return /Android/.test(ua) ? "android" : "desktop";
}

const standalone = () => matchMedia("(display-mode: standalone)").matches || (navigator as { standalone?: boolean }).standalone === true;
export const appKind = () => detectAppKind(inTauri, standalone());

const DEFAULT_NAME: Record<Platform, string> = { ios: "iPhone", android: "Android phone", desktop: "Computer" };

export const deviceName = (): string => localStorage.getItem(NAME) ?? DEFAULT_NAME[platform()];

export const setDeviceName = (name: string) => localStorage.setItem(NAME, name);

/** After removal: the next sign-in on this install registers as a new device. */
export async function resetIdentity() {
  localStorage.removeItem(ID);
  localStorage.removeItem(NAME);
  current = started = null;
  await native("reset_device_id").catch(() => {});
  await initDeviceIdentity();
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
