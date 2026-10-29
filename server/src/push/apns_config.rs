/// The APNs half of push. Built from the environment, or by tests.
pub struct ApnsConfig {
    /// PEM contents of the APNs auth key (.p8).
    pub key_pem: String,
    pub key_id: String,
    pub team_id: String,
    /// The app's bundle id.
    pub topic: String,
    /// `https://api.push.apple.com` or the sandbox host; tests point it at a fake.
    pub apns_url: String,
}

impl ApnsConfig {
    /// None (logged once) when any `APNS_*` setting is missing: the iOS app then gets no push.
    pub fn from_env() -> Option<Self> {
        let get = |key: &str| std::env::var(key).ok().filter(|v| !v.is_empty());
        let (Some(key_pem), Some(key_id), Some(team_id), Some(topic)) = (
            get("APNS_KEY"),
            get("APNS_KEY_ID"),
            get("APNS_TEAM_ID"),
            get("APNS_TOPIC"),
        ) else {
            tracing::info!("apns off: APNS_KEY, APNS_KEY_ID, APNS_TEAM_ID or APNS_TOPIC unset");
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
        })
    }
}
