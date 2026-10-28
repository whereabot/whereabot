import { invoke } from "@tauri-apps/api/core";

/**
 * supabase-js storage. The session also goes to the iOS Keychain, where the background answer
 * handler reads it and writes back a refreshed one: one refresh token for both, so a rotation
 * on one side never revokes the other. localStorage keeps a copy for every build without the
 * native plugin (desktop, Android, screenshots); the Keychain copy wins when it exists.
 */
const isSession = (key: string) => key.endsWith("-auth-token");
const call = <T>(cmd: string, args?: Record<string, unknown>) =>
  invoke<T>(`plugin:background-answer|${cmd}`, args);

export const nativeSessionStorage = {
  async getItem(key: string): Promise<string | null> {
    if (isSession(key)) {
      const res = await call<{ value: string | null } | null>("load_session").catch(() => null);
      if (typeof res?.value === "string") return res.value;
    }
    return localStorage.getItem(key);
  },
  async setItem(key: string, value: string) {
    localStorage.setItem(key, value);
    if (isSession(key)) await call("save_session", { value }).catch(() => {});
  },
  async removeItem(key: string) {
    localStorage.removeItem(key);
    if (isSession(key)) await call("remove_session").catch(() => {});
  },
};
