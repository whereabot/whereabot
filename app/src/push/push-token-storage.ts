import { deviceId } from "../devices/device-identity";
import { supabase } from "../supabase-client";

const KEY = "whereabouts.push-token";

/** Upserts this phone's APNs token for the signed-in user. Best effort. */
export async function storeToken(token: string) {
  localStorage.setItem(KEY, token);
  const { data } = await supabase.auth.getSession();
  if (!data.session) return;
  // Linked to the stable device id, so removing the device also drops its token.
  const device_id = deviceId();
  const { error } = await supabase
    .from("push_tokens")
    .upsert({ user_id: data.session.user.id, token, platform: "ios", device_id, updated_at: new Date().toISOString() });
  if (error) console.warn("push token save failed", error.message);
}

/** Sign-out: stop pushes to this phone. Needs the session, so call before signOut. */
export async function forgetToken() {
  const token = localStorage.getItem(KEY);
  localStorage.removeItem(KEY);
  if (token) await supabase.from("push_tokens").delete().eq("token", token).then(() => {}, () => {});
}
