use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{AppState, UserId};

/// The token after `Bearer ` in the Authorization header.
pub fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

/// Agent key -> user id. None rejects the request.
pub async fn agent_user(state: &AppState, headers: &HeaderMap) -> Option<Uuid> {
    state.supabase.agent_user(bearer(headers)?).await
}

/// Phone access token -> user id. None rejects the request.
pub async fn device_user(state: &AppState, headers: &HeaderMap) -> Option<Uuid> {
    state.supabase.device_user(bearer(headers)?).await
}

/// MCP gate: stores the agent's user id in the request for the tool to read.
pub async fn authorize(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(user_id) = agent_user(&state, &headers).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    request.extensions_mut().insert(UserId(user_id));
    next.run(request).await
}
