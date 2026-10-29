use super::apns_config::ApnsConfig;

/// Everything push needs. Built from the environment, or by tests.
pub struct PushConfig {
    /// None: the iOS app gets no APNs push.
    pub apns: Option<ApnsConfig>,
    /// VAPID private key: base64url of the raw 32-byte P-256 scalar. None: browsers get no Web Push.
    pub vapid_private_key: Option<String>,
    pub supabase_url: String,
    /// PostgREST needs the project's anon key next to the user's bearer.
    pub anon_key: String,
}

impl PushConfig {
    /// None (logged once) when neither APNs nor Web Push is set up: push is optional.
    pub fn from_env() -> Option<Self> {
        let get = |key: &str| std::env::var(key).ok().filter(|v| !v.is_empty());
        let apns = ApnsConfig::from_env();
        let vapid_private_key = get("VAPID_PRIVATE_KEY");
        if vapid_private_key.is_none() {
            tracing::info!("web push off: VAPID_PRIVATE_KEY unset");
        }
        if apns.is_none() && vapid_private_key.is_none() {
            return None;
        }
        let (Some(supabase_url), Some(anon_key)) = (get("SUPABASE_URL"), get("SUPABASE_ANON_KEY"))
        else {
            tracing::info!("push off: SUPABASE_URL or SUPABASE_ANON_KEY unset");
            return None;
        };
        Some(Self {
            apns,
            vapid_private_key,
            supabase_url,
            anon_key,
        })
    }
}
