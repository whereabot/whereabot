#![allow(dead_code)]
//! Fake Supabase: the JWKS endpoint and the agent-key RPC.

use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use tokio::net::TcpListener;

use super::{
    agent_rpc::{agent_key_user, key_hashes},
    jwt,
};

pub const SERVICE_KEY: &str = "service-key";
pub const USER_A: &str = "00000000-0000-4000-8000-00000000000a";
pub const USER_B: &str = "00000000-0000-4000-8000-00000000000b";

pub struct Fake {
    pub url: String,
    /// Counts JWKS requests, so tests can check the refetch rules.
    pub jwks_hits: Arc<AtomicUsize>,
    /// While set, the JWKS endpoint answers 500.
    pub jwks_fail: Arc<AtomicBool>,
    /// Device ids whose `devices` row has `revoked_at` set.
    pub revoked: Arc<Mutex<HashSet<String>>>,
}

pub struct Shared {
    pub revoked: Arc<Mutex<HashSet<String>>>,
    pub keys: HashMap<String, &'static str>,
    pub jwks_hits: Arc<AtomicUsize>,
    pub jwks_fail: Arc<AtomicBool>,
}

/// Starts the fake on a random port.
pub async fn start() -> Fake {
    let jwks_hits = Arc::new(AtomicUsize::new(0));
    let jwks_fail = Arc::new(AtomicBool::new(false));
    let revoked = Arc::new(Mutex::new(HashSet::new()));
    let shared = Arc::new(Shared {
        revoked: revoked.clone(),
        keys: key_hashes(),
        jwks_hits: jwks_hits.clone(),
        jwks_fail: jwks_fail.clone(),
    });
    let app = Router::new()
        .route("/rest/v1/rpc/agent_key_user", post(agent_key_user))
        .route("/rest/v1/devices", get(device_row))
        .route("/auth/v1/.well-known/jwks.json", get(jwks))
        .with_state(shared);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    Fake {
        url: format!("http://{addr}"),
        jwks_hits,
        jwks_fail,
        revoked,
    }
}

/// PostgREST `devices?id=eq.<id>&select=revoked_at`.
async fn device_row(
    State(shared): State<Arc<Shared>>,
    Query(q): Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let id = q["id"].trim_start_matches("eq.");
    let revoked = shared.revoked.lock().unwrap().contains(id);
    Json(serde_json::json!([{ "revoked_at": revoked.then_some("2026-10-28T00:00:00Z") }]))
}

async fn jwks(State(shared): State<Arc<Shared>>) -> Response {
    shared.jwks_hits.fetch_add(1, Ordering::SeqCst);
    if shared.jwks_fail.load(Ordering::SeqCst) {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    Json(jwt::jwks()).into_response()
}
