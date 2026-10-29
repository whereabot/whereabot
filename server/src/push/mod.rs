//! Wake-up push: an APNs actionable push the user answers from the lock screen, or taps to open the app.
//! Browser/PWA installs get the same alerts by Web Push (`web_push`).
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
mod apns_fanout;
mod apns_provider_token;
mod apns_sender;
#[cfg(test)]
mod fake_apns_and_postgrest;
#[cfg(test)]
mod fake_loopback_server;
#[cfg(test)]
mod fake_share_rpcs;
mod location_request_push_payload;
pub mod location_request_wake;
mod push_config;
mod push_token_store;
#[cfg(test)]
mod push_wake_tests;
#[cfg(test)]
mod shared_request_push_tests;
mod web_push;

use agent_grant_name::AgentNames;
use always_allowed_store::AlwaysAllowed;
use apns_sender::{Apns, Kind};
use location_request_push_payload::{Alert, payload};
pub use push_config::PushConfig;
use push_token_store::Tokens;
use web_push::{Subscriptions, WebPush};

/// How long a silent push gets to answer before the "checking" alert goes out.
const SILENT_ANSWER_WINDOW: Duration = Duration::from_secs(8);

pub struct Push {
    apns: Option<Apns>,
    web: Option<WebPush>,
    tokens: Tokens,
    subscriptions: Subscriptions,
    names: AgentNames,
    always: AlwaysAllowed,
    silent_window: Duration,
}

impl Push {
    /// None when neither APNs nor Web Push is configured and valid.
    pub fn from_env() -> Option<Self> {
        Self::new(&PushConfig::from_env()?)
    }

    pub fn new(config: &PushConfig) -> Option<Self> {
        let apns = config.apns.as_ref().and_then(Apns::new);
        let web = config.vapid_private_key.as_deref().and_then(WebPush::new);
        if apns.is_none() && web.is_none() {
            return None;
        }
        Some(Self {
            apns,
            web,
            tokens: Tokens::new(&config.supabase_url, &config.anon_key),
            subscriptions: Subscriptions::new(&config.supabase_url, &config.anon_key),
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

    /// Pushes `body` to every device of the user once: APNs, and Web Push for alerts.
    async fn send_all(&self, bearer: &str, alert: &Alert<'_>, body: &Value, kind: Kind) -> usize {
        let web = match kind {
            Kind::Alert => self.send_web(bearer, alert, body).await,
            Kind::Background => 0,
        };
        self.send_apns(bearer, alert, body, kind).await + web
    }
}
