//! APNs wake-up: an actionable push the user answers from the lock screen, or taps to open the app.
//!
//! The caller decides when to push. It spawns `wake_after` when `request_location` starts and
//! aborts the task when the request ends, so an answer inside the delay sends nothing.

use std::time::Duration;

use uuid::Uuid;

mod agent_grant_name;
mod apns_config;
mod apns_provider_token;
mod apns_sender;
#[cfg(test)]
mod fake_apns_and_postgrest;
mod location_request_push_payload;
pub mod location_request_wake;
mod push_token_store;
#[cfg(test)]
mod push_wake_tests;

use agent_grant_name::AgentNames;
pub use apns_config::PushConfig;
use apns_sender::{Apns, Sent};
use location_request_push_payload::{Alert, payload};
use push_token_store::Tokens;

pub struct Push {
    apns: Apns,
    tokens: Tokens,
    names: AgentNames,
}

impl Push {
    /// None when the `APNS_*` settings are missing or invalid.
    pub fn from_env() -> Option<Self> {
        Self::new(&PushConfig::from_env()?)
    }

    pub fn new(config: &PushConfig) -> Option<Self> {
        Some(Self {
            apns: Apns::new(config)?,
            tokens: Tokens::new(&config.supabase_url, &config.anon_key),
            names: AgentNames::new(&config.supabase_url, &config.anon_key),
        })
    }

    /// Pushes every token of the bearer's user once, and drops tokens APNs says are gone.
    /// `bearer` is the agent's Supabase user JWT. Returns how many pushes APNs accepted.
    pub async fn wake(&self, bearer: &str, alert: &Alert<'_>) -> usize {
        let tokens = self.tokens.list(bearer).await;
        if tokens.is_empty() {
            return 0;
        }
        let name = self.names.name(bearer, alert.client_id).await;
        let body = payload(alert, name.as_deref());
        let mut accepted = 0;
        for token in tokens {
            match self.apns.send(&token, &body, alert.expires_ms / 1000).await {
                Sent::Ok => accepted += 1,
                Sent::Gone => self.tokens.delete(bearer, &token).await,
                Sent::Failed => {}
            }
        }
        accepted
    }

    /// `wake` after `delay`. Abort the task to cancel it.
    pub async fn wake_after(
        &self,
        delay: Duration,
        bearer: &str,
        request_id: Uuid,
        client_id: &str,
        expires_ms: u64,
    ) {
        tokio::time::sleep(delay).await;
        let alert = Alert {
            request_id,
            client_id,
            expires_ms,
        };
        self.wake(bearer, &alert).await;
    }
}
