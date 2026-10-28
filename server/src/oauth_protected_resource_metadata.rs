//! RFC 9728 protected resource metadata (C1) and the 401 challenge that points at it (C2).
//! `public_url` may list several base URLs (comma-separated, first is canonical). Answers use
//! the one whose host the client called, so clients on an older host keep passing RFC 9728 checks.

use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{
        HeaderMap, StatusCode,
        header::{HOST, WWW_AUTHENTICATE},
    },
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};

use crate::AppState;

/// Served at `/.well-known/oauth-protected-resource` and its `/mcp` form.
pub async fn metadata(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Json<Value> {
    let base = base_url(&state.public_url, &headers);
    Json(json!({
        "resource": format!("{base}/mcp"),
        "authorization_servers": [state.issuer],
        "bearer_methods_supported": ["header"],
    }))
}

/// A 401 whose challenge names the metadata for `/mcp`.
pub fn unauthorized(public_url: &str, headers: &HeaderMap) -> Response {
    let base = base_url(public_url, headers);
    let challenge =
        format!("Bearer resource_metadata=\"{base}/.well-known/oauth-protected-resource/mcp\"");
    (StatusCode::UNAUTHORIZED, [(WWW_AUTHENTICATE, challenge)]).into_response()
}

/// The listed URL whose host matches the request's `Host`; the first one otherwise.
fn base_url<'a>(public_urls: &'a str, headers: &HeaderMap) -> &'a str {
    let host = headers.get(HOST).and_then(|h| h.to_str().ok());
    let canonical = public_urls.split(',').next().unwrap_or_default().trim();
    public_urls
        .split(',')
        .map(str::trim)
        .find(|url| host.is_some_and(|h| url.split_once("://").is_some_and(|(_, u)| u == h)))
        .unwrap_or(canonical)
}
