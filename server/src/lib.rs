use std::{collections::HashMap, sync::Arc};

use axum::{
    Router,
    routing::{any, get, post},
};
use tokio::sync::{Mutex, mpsc, oneshot};
use uuid::Uuid;

mod auth;
mod device;
mod jwks;
mod jwt;
mod locate;
mod mcp;
mod oauth;
mod protocol;
mod supabase;

use jwks::JwksCache;
use protocol::{Location, ToDevice};
use supabase::Supabase;

pub struct Config {
    pub supabase_url: String,
    /// Expected `iss` claim, normally `${supabase_url}/auth/v1`.
    pub issuer: String,
    /// Public base URL of this relay, e.g. `https://<relay-host>`.
    pub public_url: String,
    /// Service role key. `Some` turns on the headless agent-key fallback.
    pub agent_keys: Option<String>,
}

/// Owner of an authenticated agent request. Set by the MCP auth middleware.
#[derive(Clone, Copy)]
pub struct UserId(pub Uuid);

struct AppState {
    jwks: JwksCache,
    issuer: String,
    public_url: String,
    agent_keys: Option<Supabase>,
    // One live phone per user.
    devices: Mutex<HashMap<Uuid, mpsc::Sender<ToDevice>>>,
    pending: Mutex<HashMap<Uuid, oneshot::Sender<Location>>>,
}

pub fn app(config: Config) -> Router {
    let state = Arc::new(AppState {
        jwks: JwksCache::new(&config.supabase_url),
        issuer: config.issuer,
        public_url: config.public_url,
        agent_keys: config
            .agent_keys
            .map(|key| Supabase::new(config.supabase_url, key)),
        devices: Mutex::default(),
        pending: Mutex::default(),
    });
    Router::new()
        .route("/v1/locate", post(locate::locate))
        .route("/v1/device", any(device::device))
        // Cloud Run's front end reserves paths ending in `z`, so public probes use /health.
        .route("/health", get(healthz))
        .route("/healthz", get(healthz))
        .route(
            "/.well-known/oauth-protected-resource",
            get(oauth::metadata),
        )
        .route(
            "/.well-known/oauth-protected-resource/mcp",
            get(oauth::metadata),
        )
        .nest("/mcp", mcp::router(state.clone()))
        .with_state(state)
}

async fn healthz() -> &'static str {
    "ok"
}
