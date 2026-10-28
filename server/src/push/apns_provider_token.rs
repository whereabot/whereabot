//! The APNs provider token: an ES256 JWT signed with the team's .p8 key.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde_json::json;
use tokio::sync::Mutex;

/// APNs rejects a new provider token more than once per 20 minutes: keep one for 40.
const TTL: Duration = Duration::from_secs(40 * 60);

pub struct ProviderJwt {
    key: EncodingKey,
    key_id: String,
    team_id: String,
    cached: Mutex<Option<(Instant, String)>>,
}

impl ProviderJwt {
    /// None when `pem` is not an EC private key.
    pub fn new(pem: &str, key_id: &str, team_id: &str) -> Option<Self> {
        Some(Self {
            key: EncodingKey::from_ec_pem(pem.as_bytes()).ok()?,
            key_id: key_id.into(),
            team_id: team_id.into(),
            cached: Mutex::default(),
        })
    }

    pub async fn get(&self) -> Option<String> {
        let mut cached = self.cached.lock().await;
        if let Some((made, jwt)) = cached.as_ref()
            && made.elapsed() < TTL
        {
            return Some(jwt.clone());
        }
        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some(self.key_id.clone());
        let iat = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
        let claims = json!({ "iss": self.team_id, "iat": iat });
        let jwt = encode(&header, &claims, &self.key).ok()?;
        *cached = Some((Instant::now(), jwt.clone()));
        Some(jwt)
    }
}
