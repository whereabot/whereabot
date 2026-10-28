#![allow(dead_code)]
//! Shared fixtures: relay start-up, OAuth tokens, and the agent-side HTTP helpers.

pub mod agent_rpc;
pub mod jwt;
pub mod mcp;
pub mod phone;
pub mod supabase;

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize},
    },
    time::Duration,
};

use tokio::net::TcpListener;
use whereabouts_server::{Config, app};

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
}

impl Relay {
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
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    tokio::spawn(async move { axum::serve(listener, app(config)).await.unwrap() });
    Relay {
        addr,
        issuer,
        jwks_hits: fake.jwks_hits,
        jwks_fail: fake.jwks_fail,
    }
}

/// One `/v1/locate` call with the given bearer token.
pub async fn locate_once(http: &reqwest::Client, addr: &str, token: &str) -> reqwest::Response {
    http.post(format!("http://{addr}/v1/locate?timeout_secs=5"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

/// Retries `locate_once` while the phone is offline. Sockets register just after the handshake.
pub async fn locate_settled(http: &reqwest::Client, addr: &str, token: &str) -> reqwest::Response {
    let mut res = locate_once(http, addr, token).await;
    for _ in 0..50 {
        if res.status() != 503 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
        res = locate_once(http, addr, token).await;
    }
    res
}
