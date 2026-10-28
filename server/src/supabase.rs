use std::time::Duration;

use serde::de::DeserializeOwned;
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Agent-key lookup. Only built when `WB_AGENT_KEYS` is on.
#[derive(Clone)]
pub struct Supabase {
    http: reqwest::Client,
    url: String,
    service_role_key: String,
}

impl Supabase {
    pub fn new(url: String, service_role_key: String) -> Self {
        Self {
            http: http(),
            url,
            service_role_key,
        }
    }

    /// Maps a full agent key to its user. None when unknown, revoked or unreachable.
    pub async fn agent_user(&self, key: &str) -> Option<Uuid> {
        let key_hash = hex::encode(Sha256::digest(key.as_bytes()));
        let res = self
            .http
            .post(format!("{}/rest/v1/rpc/agent_key_user", self.url))
            .header("apikey", &self.service_role_key)
            .bearer_auth(&self.service_role_key)
            .json(&json!({ "p_key_hash": key_hash }))
            .send()
            .await;
        // The RPC returns the uuid as a JSON string, or null for an unknown key.
        call::<Option<Uuid>>("agent_key_user", res).await.flatten()
    }
}

/// HTTP client for Supabase calls, capped at 5 s.
pub fn http() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("http client builds")
}

/// Reads a successful JSON response. Logs the failure without any secret.
pub async fn call<T: DeserializeOwned>(
    what: &str,
    res: reqwest::Result<reqwest::Response>,
) -> Option<T> {
    let res = match res {
        Ok(res) => res,
        Err(err) => {
            tracing::warn!(what, error = %err.without_url(), "supabase request failed");
            return None;
        }
    };
    let status = res.status();
    if !status.is_success() {
        tracing::warn!(what, %status, "supabase rejected request");
        return None;
    }
    match res.json::<T>().await {
        Ok(value) => Some(value),
        Err(err) => {
            tracing::warn!(what, error = %err.without_url(), "supabase response unreadable");
            None
        }
    }
}
