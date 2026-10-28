use std::{collections::HashMap, sync::Arc};

use axum::{
    Router,
    routing::{any, get, post},
};
use tokio::sync::{Mutex, oneshot};
use uuid::Uuid;

mod auth;
mod device;
mod device_in;
mod devices;
mod errors;
mod jwks;
mod jwt;
mod locate;
mod mcp;
mod oauth;
mod protocol;
mod shutdown;
mod supabase;

use jwks::JwksCache;
use protocol::Answer;
pub use shutdown::Shutdown;
use supabase::Supabase;

pub struct Config {
    pub supabase_url: String,
    /// Expected `iss` claim, normally `${supabase_url}/auth/v1`.
    pub issuer: String,
    /// Public base URL of this relay, e.g. `https://<relay-host>`.
    pub public_url: String,
    /// Service role key. `Some` turns on the headless agent-key fallback.
    pub agent_keys: Option<String>,
    pub anon_key: Option<String>,
}

/// An authenticated agent request: whose phone to ping, and who asks. Set by the MCP auth middleware.
#[derive(Clone)]
pub struct Agent {
    pub user_id: Uuid,
    pub client_id: String,
    /// From the token's `email` claim; names the account in the offline message.
    pub email: Option<String>,
}

struct AppState {
    jwks: JwksCache,
    issuer: String,
    public_url: String,
    agent_keys: Option<Supabase>,
    supabase_url: String,
    anon_key: Option<String>,
    devices: devices::registry::Registry,
    pending: Mutex<HashMap<Uuid, oneshot::Sender<(Uuid, Answer)>>>,
}

pub fn app(config: Config) -> Router {
    build(config).0
}

/// The router plus a handle that closes device sockets on shutdown.
pub fn build(config: Config) -> (Router, Shutdown) {
    let state = Arc::new(AppState {
        jwks: JwksCache::new(&config.supabase_url),
        issuer: config.issuer,
        public_url: config.public_url,
        agent_keys: config
            .agent_keys
            .map(|key| Supabase::new(config.supabase_url.clone(), key)),
        supabase_url: config.supabase_url,
        anon_key: config.anon_key,
        devices: devices::registry::Registry::default(),
        pending: Mutex::default(),
    });
    let router = Router::new()
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
        .with_state(state.clone());
    (router, Shutdown(state))
}

async fn healthz() -> &'static str {
    "ok"
}
