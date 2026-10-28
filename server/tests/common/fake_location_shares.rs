#![allow(dead_code)]
//! Fake `location_shares` under RLS: a caller sees the rows where its token's `sub` is the grantee.

use std::sync::{Arc, Mutex};

use axum::{
    Json,
    extract::{RawQuery, State},
    http::HeaderMap,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::{Value, json};

use super::fake_supabase::Shared;

pub const USER_C: &str = "00000000-0000-4000-8000-00000000000c";

/// One share row. `revoked` stands for `revoked_at` being set.
#[derive(Clone)]
pub struct ShareRow {
    pub owner: &'static str,
    pub grantee: &'static str,
    pub name: &'static str,
    pub email: &'static str,
    pub revoked: bool,
}

pub type Shares = Arc<Mutex<Vec<ShareRow>>>;

/// The `sub` claim of the bearer token, unverified (the relay verified it already).
fn caller(headers: &HeaderMap) -> String {
    let token = headers["authorization"].to_str().unwrap();
    let body = token.split('.').nth(1).unwrap();
    let claims: Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(body).unwrap()).unwrap();
    claims["sub"].as_str().unwrap().to_owned()
}

/// `GET /rest/v1/location_shares`. Revoked rows show only when the query forgets `revoked_at=is.null`.
pub async fn select(
    State(shared): State<Arc<Shared>>,
    headers: HeaderMap,
    RawQuery(query): RawQuery,
) -> Json<Value> {
    let active_only = query.unwrap_or_default().contains("revoked_at=is.null");
    let sub = caller(&headers);
    let rows: Vec<Value> = shared
        .shares
        .lock()
        .unwrap()
        .iter()
        .filter(|row| row.grantee == sub && !(active_only && row.revoked))
        .map(|row| {
            json!({ "owner_user_id": row.owner, "owner_display_name": row.name, "owner_email": row.email })
        })
        .collect();
    Json(json!(rows))
}
