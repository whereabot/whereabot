import { supabase } from "../supabase-client";
import { type AppKind, selectToleratingMissingKind } from "./app-kind";
import { appKind, deviceId, deviceName, platform, setDeviceName } from "./device-identity";
import { currentScreenHint } from "./reinstall-device-takeover";

export type Device = {
  id: string;
  name: string;
  platform: "ios" | "android" | "desktop";
  app_kind: AppKind | null;
  created_at: string;
  last_seen_at: string;
  revoked_at: string | null;
};

const COLUMNS = "id,name,platform,app_kind,created_at,last_seen_at,revoked_at";

function check<T>(res: { data: T | null; error: { message: string } | null }): T {
  if (res.error || res.data === null) throw new Error(res.error?.message ?? "No data");
  return res.data;
}

/** Own devices that were not removed, oldest first. RLS limits rows to the signed-in user. */
export async function listDevices(): Promise<Device[]> {
  const res = await selectToleratingMissingKind(COLUMNS, (cols) =>
    supabase.from("devices").select(cols).is("revoked_at", null).order("created_at"),
  );
  return check(res as never) as Device[];
}

export async function renameDevice(id: string, name: string) {
  check((await supabase.from("devices").update({ name }).eq("id", id).select("id")) as never);
  if (id === deviceId()) setDeviceName(name);
}

/** Removing sets revoked_at; the relay then closes that device with 4000 "revoked". */
export async function removeDevice(id: string) {
  const revoked_at = new Date().toISOString();
  check((await supabase.from("devices").update({ revoked_at }).eq("id", id).select("id")) as never);
  // Revoking keeps the row, so the delete-cascade does not run: drop its push token here.
  await supabase.from("push_tokens").delete().eq("device_id", id);
}

/**
 * Registers this install on first connect, then only bumps last_seen_at.
 * The server name wins: a rename made on another device must not be overwritten.
 */
let kindSent = false;

/** Best effort and separate, so a client deployed before the app_kind/screen_hint migrations still connects. Once per page load. */
function sendAppKind(id: string) {
  if (kindSent) return;
  kindSent = true;
  void supabase.from("devices").update({ app_kind: appKind(), screen_hint: currentScreenHint() }).eq("id", id);
}

export async function syncDevice(): Promise<"ok" | "revoked"> {
  const id = deviceId();
  const last_seen_at = new Date().toISOString();
  const rows = check(
    (await supabase.from("devices").update({ last_seen_at, platform: platform() }).eq("id", id).select("name,revoked_at")) as never,
  ) as Pick<Device, "name" | "revoked_at">[];
  if (rows.length) {
    sendAppKind(id);
    setDeviceName(rows[0].name);
    return rows[0].revoked_at ? "revoked" : "ok";
  }
  check((await supabase.from("devices").insert({ id, name: deviceName(), platform: platform(), last_seen_at }).select("id")) as never);
  sendAppKind(id);
  return "ok";
}
