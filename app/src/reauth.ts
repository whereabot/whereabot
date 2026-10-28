import { supabase } from "./supabase";

let tried = false;

export const EXPIRED = "Sign-in expired. Sign out and sign in again.";
export const isAuthError = (e: unknown) => /401|403|unauthori[sz]ed|forbidden/i.test(String(e));
export const authOk = () => (tried = false);

/** Refreshes the session once per failure streak. False means it was already tried. */
export async function refreshOnce(): Promise<boolean> {
  if (tried) return false;
  tried = true;
  await supabase.auth.refreshSession().catch(() => {});
  return true;
}
