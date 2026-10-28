#![allow(dead_code)]
//! Agent keys and the fake `agent_key_user` RPC behind `WB_AGENT_KEYS`.

use std::{collections::HashMap, sync::Arc};

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::supabase::{SERVICE_KEY, Shared, USER_A, USER_B};

pub fn agent_key_a() -> String {
    format!("wb_{}", "a".repeat(48))
}

pub fn agent_key_b() -> String {
    format!("wb_{}", "b".repeat(48))
}

pub fn unknown_key() -> String {
    format!("wb_{}", "c".repeat(48))
}

/// Key hash -> user, as the RPC stores them.
pub fn key_hashes() -> HashMap<String, &'static str> {
    HashMap::from([
        (sha256_hex(&agent_key_a()), USER_A),
        (sha256_hex(&agent_key_b()), USER_B),
    ])
}

fn sha256_hex(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))
}

fn header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name)?.to_str().ok()
}

pub async fn agent_key_user(
    State(shared): State<Arc<Shared>>,
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
    Json(
        shared
            .keys
            .get(hash)
            .map_or(Value::Null, |user| json!(user)),
    )
    .into_response()
}
