//! APNs wake-up: an actionable push the user answers from the lock screen, or taps to open the app.
//! Always-allowed agents get a silent push first; see `always_allowed_wake`.
//!
//! The caller decides when to push. It spawns `wake_after` when `request_location` starts and
//! aborts the task when the request ends, so an answer inside the delay sends nothing.

use std::time::Duration;

use serde_json::Value;

mod agent_grant_name;
mod always_allowed_store;
mod always_allowed_wake;
#[cfg(test)]
mod always_allowed_wake_tests;
mod apns_config;
mod apns_provider_token;
mod apns_sender;
#[cfg(test)]
mod fake_apns_and_postgrest;
#[cfg(test)]
mod fake_share_rpcs;
mod location_request_push_payload;
pub mod location_request_wake;
mod push_token_store;
#[cfg(test)]
mod push_wake_tests;
#[cfg(test)]
mod shared_request_push_tests;

use agent_grant_name::AgentNames;
use always_allowed_store::AlwaysAllowed;
pub use apns_config::PushConfig;
use apns_sender::{Apns, Kind, Sent};
use location_request_push_payload::{Alert, payload};
use push_token_store::Tokens;

/// How long a silent push gets to answer before the "checking" alert goes out.
const SILENT_ANSWER_WINDOW: Duration = Duration::from_secs(8);

pub struct Push {
    apns: Apns,
    tokens: Tokens,
    names: AgentNames,
    always: AlwaysAllowed,
    silent_window: Duration,
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
            always: AlwaysAllowed::new(&config.supabase_url, &config.anon_key),
            silent_window: SILENT_ANSWER_WINDOW,
        })
    }

    /// The consent alert ("wants to know"). `bearer` is the agent's Supabase user JWT.
    /// Returns how many pushes APNs accepted.
    pub async fn wake(&self, bearer: &str, alert: &Alert<'_>) -> usize {
        let name = self.names.name(bearer, alert.client_id).await;
        let body = payload(alert, name.as_deref());
        self.send_all(bearer, alert, &body, Kind::Alert).await
    }

    /// Pushes `body` to every token of the bearer's user once, and drops tokens APNs says are gone.
    async fn send_all(&self, bearer: &str, alert: &Alert<'_>, body: &Value, kind: Kind) -> usize {
        let tokens = self.tokens.list(bearer, alert.owner).await;
        if tokens.is_empty() {
            tracing::info!(request_id = %alert.request_id, "push skipped: no push token for this user");
            return 0;
        }
        let mut accepted = 0;
        let total = tokens.len();
        for token in tokens {
            match self
                .apns
                .send(&token, body, kind, alert.expires_ms / 1000)
                .await
            {
                Sent::Ok => accepted += 1,
                // shortcut: a requester's JWT cannot delete the owner's token; the owner's own next request does.
                Sent::Gone if alert.owner.is_none() => self.tokens.delete(bearer, &token).await,
                Sent::Gone => {}
                Sent::Failed => {}
            }
        }
        tracing::info!(request_id = %alert.request_id, total, accepted, "push sent");
        accepted
    }
}
