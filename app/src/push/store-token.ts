import { supabase } from "../supabase";

const KEY = "whereabouts.push-token";

/** Upserts this phone's APNs token for the signed-in user. Best effort. */
export async function storeToken(token: string) {
  localStorage.setItem(KEY, token);
  const { data } = await supabase.auth.getSession();
  if (!data.session) return;
  // `wb.device_id` is written by the devices feature; null until this install has one.
  const device_id = localStorage.getItem("wb.device_id");
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
