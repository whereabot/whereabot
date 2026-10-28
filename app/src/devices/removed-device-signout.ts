import { signOut } from "../supabase-sign-in";
import { setStatus } from "../status-display-dom";
import { clearLocalData } from "../request-history-store";
import { resetIdentity } from "./device-identity";

export const REMOVED = "This device was removed from your account. Sign in again to add it back.";

/**
 * This install was removed: forget it and sign out. The auth listener in main.ts
 * disconnects. Its "Disconnected." status lands first, so the reason is set after it.
 */
export async function signOutRemoved(message = REMOVED) {
  await resetIdentity();
  clearLocalData();
  await signOut();
  window.setTimeout(() => setStatus(message), 100);
}
