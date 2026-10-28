//! APNs wake-up: a push that opens the app so a waiting location request can reach it.
//!
//! The caller decides when to push. It spawns `wake_after` when `request_location` starts and
//! aborts the task when the request ends, so an answer inside the delay sends nothing.

use std::time::Duration;

use uuid::Uuid;

mod apns_config;
mod apns_provider_token;
mod apns_sender;
#[cfg(test)]
mod fake_apns_and_postgrest;
pub mod location_request_wake;
mod push_token_store;
#[cfg(test)]
mod push_wake_tests;

pub use apns_config::PushConfig;
use apns_sender::{Apns, Sent};
use push_token_store::Tokens;

pub struct Push {
    apns: Apns,
    tokens: Tokens,
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
        })
    }

    /// Pushes every token of the bearer's user once, and drops tokens APNs says are gone.
    /// `bearer` is the agent's Supabase user JWT. Returns how many pushes APNs accepted.
    pub async fn wake(&self, bearer: &str, request_id: Uuid, expires_ms: u64) -> usize {
        let mut accepted = 0;
        for token in self.tokens.list(bearer).await {
            match self.apns.send(&token, request_id, expires_ms / 1000).await {
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
        expires_ms: u64,
    ) {
        tokio::time::sleep(delay).await;
        self.wake(bearer, request_id, expires_ms).await;
    }
}
