//! The user's always-allowed agents in Supabase (`always_allowed_agents`). Read with the
//! agent's own user JWT: RLS lets it read its user's rows, never write them. For someone
//! else's phones the `is_always_allowed_for_requester` RPC answers instead.

use serde_json::{Value, json};
use uuid::Uuid;

use crate::supabase_rest_client;

pub struct AlwaysAllowed {
    http: reqwest::Client,
    url: String,
    rpc: String,
    anon_key: String,
}

impl AlwaysAllowed {
    pub fn new(supabase_url: &str, anon_key: &str) -> Self {
        Self {
            http: supabase_rest_client::http(),
            url: format!("{supabase_url}/rest/v1/always_allowed_agents"),
            rpc: format!("{supabase_url}/rest/v1/rpc/is_always_allowed_for_requester"),
            anon_key: anon_key.into(),
        }
    }

    /// False when the lookup fails: the user then gets the normal consent alert.
    /// With `owner`, asks whether that owner always allows this client for the bearer's user.
    pub async fn contains(&self, bearer: &str, client_id: &str, owner: Option<Uuid>) -> bool {
        if let Some(owner) = owner {
            let res = self
                .http
                .post(&self.rpc)
                .header("apikey", &self.anon_key)
                .bearer_auth(bearer)
                .json(&json!({ "p_owner": owner, "p_client_id": client_id }))
                .send()
                .await;
            return supabase_rest_client::call::<bool>("is_always_allowed_for_requester", res)
                .await
                .unwrap_or(false);
        }
        let Ok(mut url) = reqwest::Url::parse(&self.url) else {
            return false;
        };
        // Encoded: a client id is free text from the OAuth server.
        url.query_pairs_mut()
            .append_pair("select", "client_id")
            .append_pair("client_id", &format!("eq.{client_id}"))
            // The owner's own agent only; rows for requesters do not count here.
            .append_pair("requester_user_id", "is.null");
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
