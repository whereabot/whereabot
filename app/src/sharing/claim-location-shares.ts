import { supabase } from "../supabase-client";

const claim = () => void supabase.rpc("claim_location_shares").then(({ error }) => error && console.warn("claim shares failed", error.message));

/** On sign-in, links shares made to this verified email to this account. Best effort. */
export function initClaimLocationShares() {
  supabase.auth.onAuthStateChange((event) => {
    // Deferred: supabase calls must not run inside the auth callback.
    if (event === "SIGNED_IN" || event === "INITIAL_SESSION") setTimeout(claim, 0);
  });
}
