use std::sync::{Arc, Mutex};

use axum::{
    Router,
    routing::{any, get, post},
};
mod agent_bearer_auth;
mod agent_token_verification;
mod authenticated_agent;
mod background_answer_http;
mod device_socket_session;
mod devices;
mod list_people_tool;
mod locate_outcome_log;
mod locate_rate_limit;
mod location_request;
mod location_request_error;
mod location_request_http;
mod location_target;
mod mcp_http_route;
mod mcp_location_tool;
mod oauth_protected_resource_metadata;
mod pending_location_requests;
mod phone_message_handler;
pub mod push;
mod relay_wire_protocol;
mod shared_person_resolution;
mod supabase_signing_keys;
pub use authenticated_agent::Agent;
pub use locate_rate_limit::LocateLimits;
pub use mcp_location_tool::tool_list_json;
mod relay_config;
pub use relay_config::Config;
pub use relay_wire_protocol::{FromDevice, ToDevice};
mod graceful_shutdown;
mod supabase_rest_client;
pub use graceful_shutdown::Shutdown;
use supabase_rest_client::Supabase;
use supabase_signing_keys::JwksCache;

struct AppState {
    push: Option<Arc<push::Push>>,
    jwks: JwksCache,
    issuer: String,
    public_url: String,
    agent_keys: Option<Supabase>,
    supabase_url: String,
    anon_key: Option<String>,
    devices: devices::device_connection_registry::Registry,
    pending: Mutex<pending_location_requests::Pending>,
    locate_limiter: locate_rate_limit::LocateLimiter,
}

pub fn app(config: Config) -> Router {
    build(config).0
}

/// The router plus a handle that closes device sockets on shutdown.
pub fn build(config: Config) -> (Router, Shutdown) {
    let state = Arc::new(AppState {
        push: push::Push::from_env().map(Arc::new),
        jwks: JwksCache::new(&config.supabase_url),
        issuer: config.issuer,
        public_url: config.public_url,
        agent_keys: config
            .agent_keys
            .map(|key| Supabase::new(config.supabase_url.clone(), key)),
        supabase_url: config.supabase_url,
        anon_key: config.anon_key,
        devices: devices::device_connection_registry::Registry::default(),
        pending: Mutex::default(),
        locate_limiter: locate_rate_limit::LocateLimiter::new(config.locate_limits),
    });
    let router = Router::new()
        .route("/v1/locate", post(location_request_http::locate))
        .route("/v1/device", any(device_socket_session::device))
        .route("/v1/device/answer", post(background_answer_http::answer))
        // Cloud Run's front end reserves paths ending in `z`, so public probes use /health.
        .route("/health", get(healthz))
        .route("/healthz", get(healthz))
        .route(
            "/.well-known/oauth-protected-resource",
            get(oauth_protected_resource_metadata::metadata),
        )
        .route(
            "/.well-known/oauth-protected-resource/mcp",
            get(oauth_protected_resource_metadata::metadata),
        )
        .nest("/mcp", mcp_http_route::router(state.clone()))
        .with_state(state.clone());
    (router, Shutdown(state))
}

async fn healthz() -> &'static str {
    "ok"
}
