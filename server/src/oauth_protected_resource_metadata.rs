//! RFC 9728 protected resource metadata (C1) and the 401 challenge that points at it (C2).

use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{StatusCode, header::WWW_AUTHENTICATE},
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};

use crate::AppState;

/// Served at `/.well-known/oauth-protected-resource` and its `/mcp` form.
pub async fn metadata(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({
        "resource": format!("{}/mcp", state.public_url),
        "authorization_servers": [state.issuer],
        "bearer_methods_supported": ["header"],
    }))
}

/// A 401 whose challenge names the metadata for `/mcp`.
pub fn unauthorized(public_url: &str) -> Response {
    let challenge = format!(
        "Bearer resource_metadata=\"{public_url}/.well-known/oauth-protected-resource/mcp\""
    );
    (StatusCode::UNAUTHORIZED, [(WWW_AUTHENTICATE, challenge)]).into_response()
}
