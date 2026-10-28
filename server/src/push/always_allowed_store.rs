//! The user's always-allowed agents in Supabase (`always_allowed_agents`). Read with the
//! agent's own user JWT: RLS lets it read its user's rows, never write them.

use serde_json::Value;

use crate::supabase_rest_client;

pub struct AlwaysAllowed {
    http: reqwest::Client,
    url: String,
    anon_key: String,
}

impl AlwaysAllowed {
    pub fn new(supabase_url: &str, anon_key: &str) -> Self {
        Self {
            http: supabase_rest_client::http(),
            url: format!("{supabase_url}/rest/v1/always_allowed_agents"),
            anon_key: anon_key.into(),
        }
    }

    /// False when the lookup fails: the user then gets the normal consent alert.
    pub async fn contains(&self, bearer: &str, client_id: &str) -> bool {
        let Ok(mut url) = reqwest::Url::parse(&self.url) else {
            return false;
        };
        // Encoded: a client id is free text from the OAuth server.
        url.query_pairs_mut()
            .append_pair("select", "client_id")
            .append_pair("client_id", &format!("eq.{client_id}"));
        let res = self
            .http
            .get(url)
            .header("apikey", &self.anon_key)
            .bearer_auth(bearer)
            .send()
            .await;
        supabase_rest_client::call::<Vec<Value>>("always_allowed_agents select", res)
            .await
            .is_some_and(|rows| !rows.is_empty())
    }
}
