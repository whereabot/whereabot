//! Test double for the sharing RPCs: `OWNER` shares with the caller, has one token `cc33`,
//! and always allows `trusted-agent` for that caller.

use std::sync::Arc;

use axum::{Json, Router, routing::post};
use serde_json::{Value, json};

use super::fake_apns_and_postgrest::Fake;

pub const OWNER: &str = "00000000-0000-4000-8000-0000000000c0";

async fn tokens(Json(body): Json<Value>) -> Json<Value> {
    Json(if body["p_owner"] == OWNER {
        json!(["cc33"])
    } else {
        json!([])
    })
}

async fn always(Json(body): Json<Value>) -> Json<Value> {
    Json(json!(
        body["p_owner"] == OWNER && body["p_client_id"] == "trusted-agent"
    ))
}

pub fn routes() -> Router<Arc<Fake>> {
    Router::new()
        .route("/rest/v1/rpc/share_target_push_tokens", post(tokens))
        .route("/rest/v1/rpc/is_always_allowed_for_requester", post(always))
}
