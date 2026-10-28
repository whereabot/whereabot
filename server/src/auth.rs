use axum::{
    extract::{Request, State},
    http::{HeaderMap, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{AppState, UserId, jwt, oauth};

/// The token after `Bearer ` in the Authorization header.
pub fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

/// Supabase access token (C3) -> user id. `require_client_id` is set for MCP, not the phone.
pub async fn verify(
    state: &AppState,
    headers: &HeaderMap,
    require_client_id: bool,
) -> Option<Uuid> {
    let token = bearer(headers)?;
    let kid = jsonwebtoken::decode_header(token).ok()?.kid?;
    let key = state.jwks.key(&kid).await?;
    jwt::check(token, &key, &state.issuer, require_client_id)
}

/// Caller of `/mcp` and `/v1/locate`: an OAuth client token, or an agent key when enabled.
pub async fn agent_user(state: &AppState, headers: &HeaderMap) -> Option<Uuid> {
    if let Some(user_id) = verify(state, headers, true).await {
        return Some(user_id);
    }
    state
        .agent_keys
        .as_ref()?
        .agent_user(bearer(headers)?)
        .await
}

/// MCP gate: stores the caller's user id in the request for the tool to read.
pub async fn authorize(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(user_id) = agent_user(&state, &headers).await else {
        return oauth::unauthorized(&state.public_url);
    };
    request.extensions_mut().insert(UserId(user_id));
    next.run(request).await
}
