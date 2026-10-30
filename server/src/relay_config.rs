//! Settings `main` reads from the environment and hands to `build`.

use crate::LocateLimits;

pub struct Config {
    pub supabase_url: String,
    /// Expected `iss` claim, normally `${supabase_url}/auth/v1`.
    pub issuer: String,
    /// `WB_PUBLIC_URL`: base URL(s) of this relay, comma-separated; the first is canonical.
    pub public_url: String,
    /// Service role key. `Some` turns on the headless agent-key fallback.
    pub agent_keys: Option<String>,
    pub anon_key: Option<String>,
    pub locate_limits: LocateLimits,
}
