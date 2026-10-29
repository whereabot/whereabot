//! The user's Web Push subscriptions in Supabase. Called with the agent's own user JWT, so RLS
//! scopes every row to that user; a shared request reads the owner's through a share-checked RPC.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use web_push_native::{Auth, WebPushBuilder, p256::PublicKey};

use super::web_push_endpoint_allowlist::trusted;
use crate::supabase_rest_client;

/// One `PushSubscription` as the browser gave it (keys base64url).
#[derive(Deserialize)]
pub struct Subscription {
    /// Absent from the share RPC: a requester never deletes the owner's rows.
    #[serde(default)]
    pub id: Option<Uuid>,
    pub endpoint: String,
    p256dh: String,
    auth: String,
}

impl Subscription {
    /// None when the row is not a valid subscription on a known push service.
    pub fn builder(&self) -> Option<WebPushBuilder> {
        let url = self.endpoint.parse().ok()?;
        if !trusted(&url) {
            return None;
        }
        let decode = |s: &str| URL_SAFE_NO_PAD.decode(s.trim_end_matches('=')).ok();
        let key = PublicKey::from_sec1_bytes(&decode(&self.p256dh)?).ok()?;
        let auth = decode(&self.auth).filter(|a| a.len() == 16)?;
        let uri = self.endpoint.parse().ok()?;
        Some(WebPushBuilder::new(uri, key, Auth::clone_from_slice(&auth)))
    }
}

pub struct Subscriptions {
    http: reqwest::Client,
    url: String,
    rpc: String,
    anon_key: String,
}

impl Subscriptions {
    pub fn new(supabase_url: &str, anon_key: &str) -> Self {
        Self {
            http: supabase_rest_client::http(),
            url: format!("{supabase_url}/rest/v1/web_push_subscriptions"),
            rpc: format!("{supabase_url}/rest/v1/rpc/share_target_web_push_subscriptions"),
            anon_key: anon_key.into(),
        }
    }

    /// The bearer's own subscriptions, or with `owner` those of someone who shares with the bearer.
    pub async fn list(&self, bearer: &str, owner: Option<Uuid>) -> Vec<Subscription> {
        let res = match owner {
            Some(owner) => self.http.post(&self.rpc).json(&json!({ "p_owner": owner })),
            None => self
                .http
                .get(format!("{}?select=id,endpoint,p256dh,auth", self.url)),
        };
        let res = res
            .header("apikey", &self.anon_key)
            .bearer_auth(bearer)
            .send()
            .await;
        supabase_rest_client::call("web_push_subscriptions select", res)
            .await
            .unwrap_or_default()
    }

    pub async fn delete(&self, bearer: &str, id: Uuid) {
        let res = self
            .http
            .delete(format!("{}?id=eq.{id}", self.url))
            .header("apikey", &self.anon_key)
            .bearer_auth(bearer)
            .send()
            .await;
        if let Err(err) = res {
            tracing::warn!(error = %err.without_url(), "web push subscription delete failed");
        }
    }
}
