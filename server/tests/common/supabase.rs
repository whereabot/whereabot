#![allow(dead_code)]
//! Fake Supabase: the agent-key RPC and the auth user endpoint.

use std::{collections::HashMap, sync::Arc};

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::net::TcpListener;

pub const ANON_KEY: &str = "anon-key";
pub const SERVICE_KEY: &str = "service-key";
pub const USER_A: &str = "00000000-0000-4000-8000-00000000000a";
pub const USER_B: &str = "00000000-0000-4000-8000-00000000000b";

pub fn agent_key_a() -> String {
    format!("wb_{}", "a".repeat(48))
}

pub fn agent_key_b() -> String {
    format!("wb_{}", "b".repeat(48))
}

pub fn unknown_key() -> String {
    format!("wb_{}", "c".repeat(48))
}

type Keys = Arc<HashMap<String, &'static str>>;

/// Starts the fake on a random port and returns its base URL.
pub async fn start() -> String {
    let keys: Keys = Arc::new(HashMap::from([
        (sha256_hex(&agent_key_a()), USER_A),
        (sha256_hex(&agent_key_b()), USER_B),
    ]));
    let app = Router::new()
        .route("/rest/v1/rpc/agent_key_user", post(agent_key_user))
        .route("/auth/v1/user", get(auth_user))
        .with_state(keys);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

fn sha256_hex(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))
}

fn header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name)?.to_str().ok()
}

async fn agent_key_user(
    State(keys): State<Keys>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    let bearer = format!("Bearer {SERVICE_KEY}");
    if header(&headers, "apikey") != Some(SERVICE_KEY)
        || header(&headers, "authorization") != Some(bearer.as_str())
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let hash = body["p_key_hash"].as_str().unwrap_or_default();
    Json(keys.get(hash).map_or(Value::Null, |user| json!(user))).into_response()
}

async fn auth_user(headers: HeaderMap) -> Response {
    if header(&headers, "apikey") != Some(ANON_KEY) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let user = match header(&headers, "authorization").and_then(|v| v.strip_prefix("Bearer ")) {
        Some("phone-a") => USER_A,
        Some("phone-b") => USER_B,
        _ => return StatusCode::UNAUTHORIZED.into_response(),
    };
    Json(json!({ "id": user })).into_response()
}
