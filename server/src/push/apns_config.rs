/// Everything push needs. Built from the environment, or by tests.
pub struct PushConfig {
    /// PEM contents of the APNs auth key (.p8).
    pub key_pem: String,
    pub key_id: String,
    pub team_id: String,
    /// The app's bundle id.
    pub topic: String,
    /// `https://api.push.apple.com` or the sandbox host; tests point it at a fake.
    pub apns_url: String,
    pub supabase_url: String,
    /// PostgREST needs the project's anon key next to the user's bearer.
    pub anon_key: String,
}

impl PushConfig {
    /// None (logged once) when any setting is missing: push is optional.
    pub fn from_env() -> Option<Self> {
        let get = |key: &str| std::env::var(key).ok().filter(|v| !v.is_empty());
        let (Some(key_pem), Some(key_id), Some(team_id), Some(topic)) = (
            get("APNS_KEY"),
            get("APNS_KEY_ID"),
            get("APNS_TEAM_ID"),
            get("APNS_TOPIC"),
        ) else {
            tracing::info!("push off: APNS_KEY, APNS_KEY_ID, APNS_TEAM_ID or APNS_TOPIC unset");
            return None;
        };
        let (Some(supabase_url), Some(anon_key)) = (get("SUPABASE_URL"), get("SUPABASE_ANON_KEY"))
        else {
            tracing::info!("push off: SUPABASE_URL or SUPABASE_ANON_KEY unset");
            return None;
        };
        let production = matches!(get("APNS_PRODUCTION").as_deref(), Some("true" | "1"));
        let host = if production {
            "api.push.apple.com"
        } else {
            "api.development.push.apple.com"
        };
        Some(Self {
            // Env stores may flatten the PEM's newlines to a literal `\n`.
            key_pem: key_pem.replace("\\n", "\n"),
            key_id,
            team_id,
            topic,
            apns_url: format!("https://{host}"),
            supabase_url,
            anon_key,
        })
    }
}
