import { deviceId } from "../devices/device-identity";
import { supabase } from "../supabase-client";

// Browser build only. Empty key: the build has no Web Push (infra/README.md, "Add the VAPID key").
const vapidKey = import.meta.env.VITE_VAPID_PUBLIC_KEY ?? "";
// iOS Safari has PushManager only in a Home Screen install (16.4+).
const supported = () => !!vapidKey && "serviceWorker" in navigator && "PushManager" in window;

/** The VAPID public key as PushManager wants it: base64url → bytes. */
function keyBytes(b64: string): Uint8Array<ArrayBuffer> {
  const bin = atob(b64.replace(/-/g, "+").replace(/_/g, "/"));
  return Uint8Array.from(bin, (c) => c.charCodeAt(0));
}

async function save(userId: string, sub: PushSubscriptionJSON, device_id: string | null) {
  const row = { user_id: userId, device_id, endpoint: sub.endpoint, p256dh: sub.keys?.p256dh, auth: sub.keys?.auth };
  return supabase.from("web_push_subscriptions").upsert(row, { onConflict: "endpoint" });
}

/** Subscribes this browser and stores the subscription for the signed-in user. Best effort. */
export async function registerWebPush() {
  try {
    const { data } = await supabase.auth.getSession();
    if (!supported() || !data.session || Notification.permission !== "granted") return;
    const reg = await navigator.serviceWorker.ready;
    // shortcut: a subscription made with an older VAPID key is kept; unsubscribe on key rotation.
    const sub =
      (await reg.pushManager.getSubscription()) ??
      (await reg.pushManager.subscribe({ userVisibleOnly: true, applicationServerKey: keyBytes(vapidKey) }));
    const json = sub.toJSON();
    let { error } = await save(data.session.user.id, json, deviceId());
    // 23503: the relay has not seen this install's hello yet, so no devices row; link it next time.
    if (error?.code === "23503") ({ error } = await save(data.session.user.id, json, null));
    if (error) console.warn("web push save failed", error.message);
  } catch (e) {
    console.warn("web push registration failed", e);
  }
}

/** Sign-out: stop pushes to this browser. Needs the session, so call before signOut. */
export async function forgetWebPush() {
  if (!supported()) return;
  const reg = await navigator.serviceWorker.getRegistration().catch(() => undefined);
  const sub = await reg?.pushManager.getSubscription().catch(() => null);
  if (!sub) return;
  await supabase.from("web_push_subscriptions").delete().eq("endpoint", sub.endpoint).then(() => {}, () => {});
  await sub.unsubscribe().catch(() => false);
}

/** The user may allow notifications on the permissions screen, with no reload or focus change. */
export function onNotificationPermissionChange(run: () => void) {
  if (!supported() || !navigator.permissions) return;
  void navigator.permissions.query({ name: "notifications" }).then((p) => (p.onchange = run), () => {});
}
