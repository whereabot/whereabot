#![allow(dead_code)]
//! Shared fixtures: relay start-up, OAuth tokens, and the agent-side HTTP helpers.

pub mod agent_rpc;
pub mod http;
pub mod jwt;
pub mod mcp;
pub mod phone;
pub mod scripted;
pub mod supabase;

#[allow(unused_imports)]
pub use http::{locate_once, locate_settled};

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize},
};

use tokio::net::TcpListener;
use whereabouts_server::{Config, Shutdown, build};

pub const PUBLIC_URL: &str = "https://relay.test";
/// The `WWW-Authenticate` value every 401 on `/mcp` and `/v1/device` must carry (C2).
pub const CHALLENGE: &str =
    "Bearer resource_metadata=\"https://relay.test/.well-known/oauth-protected-resource/mcp\"";

pub struct Relay {
    pub addr: String,
    pub issuer: String,
    pub jwks_hits: Arc<AtomicUsize>,
    /// Set to make the fake JWKS endpoint answer 500.
    pub jwks_fail: Arc<AtomicBool>,
    /// Device ids the fake `devices` table marks revoked.
    pub revoked: Arc<std::sync::Mutex<std::collections::HashSet<String>>>,
    /// Closes device sockets, as SIGTERM does.
    pub shutdown: Shutdown,
}

impl Relay {
    /// An MCP token that also carries an `email` claim.
    pub fn mcp_email(&self, user: &str, email: &str) -> String {
        let mut claims = jwt::claims(&self.issuer, user, true);
        claims["email"] = email.into();
        jwt::sign(&claims)
    }

    /// A token an MCP client holds: has `client_id`.
    pub fn mcp(&self, user: &str) -> String {
        jwt::sign(&jwt::claims(&self.issuer, user, true))
    }

    /// The phone's session token: no `client_id`.
    pub fn phone(&self, user: &str) -> String {
        jwt::sign(&jwt::claims(&self.issuer, user, false))
    }
}

/// Starts a fake Supabase and a relay wired to it, with the agent-key fallback off.
pub async fn start_relay() -> Relay {
    spawn(false).await
}

/// Starts a relay with `WB_AGENT_KEYS` on.
pub async fn spawn(agent_keys: bool) -> Relay {
    let fake = supabase::start().await;
    let issuer = format!("{}/auth/v1", fake.url);
    let config = Config {
        supabase_url: fake.url.clone(),
        issuer: issuer.clone(),
        public_url: PUBLIC_URL.into(),
        agent_keys: agent_keys.then(|| supabase::SERVICE_KEY.into()),
        anon_key: None,
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let (router, shutdown) = build(config);
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    Relay {
        shutdown,
        addr,
        issuer,
        jwks_hits: fake.jwks_hits,
        jwks_fail: fake.jwks_fail,
        revoked: fake.revoked,
    }
}
