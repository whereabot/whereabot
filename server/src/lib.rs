use std::{collections::HashMap, sync::Arc};

use axum::{
    Router,
    routing::{any, get, post},
};
use tokio::sync::{Mutex, mpsc, oneshot};
use uuid::Uuid;

mod auth;
mod device;
mod locate;
mod mcp;
mod protocol;
mod supabase;

use protocol::{Location, ToDevice};
use supabase::Supabase;

pub struct Config {
    pub supabase_url: String,
    pub anon_key: String,
    pub service_role_key: String,
}

/// Owner of an authenticated agent request. Set by the MCP auth middleware.
#[derive(Clone, Copy)]
pub struct UserId(pub Uuid);

struct AppState {
    supabase: Supabase,
    // One live phone per user.
    devices: Mutex<HashMap<Uuid, mpsc::Sender<ToDevice>>>,
    pending: Mutex<HashMap<Uuid, oneshot::Sender<Location>>>,
}

pub fn app(config: Config) -> Router {
    let state = Arc::new(AppState {
        supabase: Supabase::new(
            config.supabase_url,
            config.anon_key,
            config.service_role_key,
        ),
        devices: Mutex::default(),
        pending: Mutex::default(),
    });
    Router::new()
        .route("/v1/locate", post(locate::locate))
        .route("/v1/device", any(device::device))
        // Cloud Run's front end reserves paths ending in `z`, so public probes use /health.
        .route("/health", get(healthz))
        .route("/healthz", get(healthz))
        .nest("/mcp", mcp::router(state.clone()))
        .with_state(state)
}

async fn healthz() -> &'static str {
    "ok"
}
