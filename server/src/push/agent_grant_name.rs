//! The agent's display name for the push text: the OAuth client name the user approved.
//! Read from the user's own grants with the agent's user JWT, the same list the app shows.

use serde::Deserialize;

use crate::supabase_rest_client;

#[derive(Deserialize)]
struct Grant {
    client: Client,
}

#[derive(Deserialize)]
struct Client {
    id: String,
    name: Option<String>,
}

pub struct AgentNames {
    http: reqwest::Client,
    url: String,
    anon_key: String,
}

impl AgentNames {
    pub fn new(supabase_url: &str, anon_key: &str) -> Self {
        Self {
            http: supabase_rest_client::http(),
            url: format!("{supabase_url}/auth/v1/user/oauth/grants"),
            anon_key: anon_key.into(),
        }
    }

    /// None when the lookup fails or the client has no name: the push then says "An agent".
    pub async fn name(&self, bearer: &str, client_id: &str) -> Option<String> {
        let res = self
            .http
            .get(&self.url)
            .header("apikey", &self.anon_key)
            .bearer_auth(bearer)
            .send()
            .await;
        supabase_rest_client::call::<Vec<Grant>>("oauth grants", res)
            .await?
            .into_iter()
            .find(|grant| grant.client.id == client_id)?
            .client
            .name
            .filter(|name| !name.trim().is_empty())
    }
}
