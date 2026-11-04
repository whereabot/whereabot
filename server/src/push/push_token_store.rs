//! The user's APNs tokens in Supabase. Called with the agent's own user JWT, so RLS scopes every row to that user.

use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::push_target_device::{DeviceRow, Token, not_revoked};
use crate::supabase_rest_client;

#[derive(Deserialize)]
struct Row {
    token: String,
    /// The install the token belongs to; null for tokens saved before devices existed.
    devices: Option<DeviceRow>,
}

pub struct Tokens {
    http: reqwest::Client,
    url: String,
    rpc: String,
    anon_key: String,
}

impl Tokens {
    pub fn new(supabase_url: &str, anon_key: &str) -> Self {
        Self {
            http: supabase_rest_client::http(),
            url: format!("{supabase_url}/rest/v1/push_tokens"),
            rpc: format!("{supabase_url}/rest/v1/rpc/share_target_push_tokens"),
            anon_key: anon_key.into(),
        }
    }

    /// The bearer's own tokens, or with `owner` the tokens of someone who shares with the
    /// bearer's user. The `share_target_push_tokens` RPC checks that share.
    pub async fn list(&self, bearer: &str, owner: Option<Uuid>) -> Vec<Token> {
        if let Some(owner) = owner {
            let res = self
                .http
                .post(&self.rpc)
                .header("apikey", &self.anon_key)
                .bearer_auth(bearer)
                .json(&json!({ "p_owner": owner }))
                .send()
                .await;
            // shortcut: the RPC returns bare tokens, so a shared request treats each as linked; return device_id from it if that matters.
            return supabase_rest_client::call::<Vec<String>>("share_target_push_tokens", res)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|token| Token {
                    token,
                    linked: true,
                })
                .collect();
        }
        let res = self
            .http
            .get(format!(
                "{}?select=token,devices(revoked_at)&platform=eq.ios",
                self.url
            ))
            .header("apikey", &self.anon_key)
            .bearer_auth(bearer)
            .send()
            .await;
        supabase_rest_client::call::<Vec<Row>>("push_tokens select", res)
            .await
            .unwrap_or_default()
            .into_iter()
            .filter(|row| not_revoked(row.devices.as_ref()))
            .map(|row| Token {
                linked: row.devices.is_some(),
                token: row.token,
            })
            .collect()
    }

    pub async fn delete(&self, bearer: &str, token: &str) {
        // Only hex reaches the URL: the app stores APNs hex tokens.
        if !token.bytes().all(|b| b.is_ascii_hexdigit()) {
            return;
        }
        let res = self
            .http
            .delete(format!("{}?token=eq.{token}", self.url))
            .header("apikey", &self.anon_key)
            .bearer_auth(bearer)
            .send()
            .await;
        if let Err(err) = res {
            tracing::warn!(error = %err.without_url(), "push token delete failed");
        }
    }
}
