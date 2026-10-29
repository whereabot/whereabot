import { supabase } from "../supabase-client";
import { adoptDeviceId, appKind, deviceId, platform } from "./device-identity";
import { rowToTakeOver, screenHint } from "./reinstall-device-match";

export const currentScreenHint = () => screenHint(screen.width, screen.height, devicePixelRatio);

let checked = false;

/**
 * Once per page load, before the hello: a reinstalled app or PWA takes over its old devices row
 * instead of adding one. RLS returns only the signed-in user's rows, so no one adopts another
 * user's device. Best effort: on any error this install keeps its own id.
 */
export async function takeOverOldDeviceRow() {
  if (checked) return;
  checked = true;
  const { data, error } = await supabase.from("devices").select("id,platform,app_kind,screen_hint,last_seen_at,revoked_at");
  if (error || !data) return;
  const me = { id: deviceId(), platform: platform(), appKind: appKind(), screenHint: currentScreenHint() };
  const id = rowToTakeOver(data, me);
  if (id) await adoptDeviceId(id);
}
