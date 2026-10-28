use axum::{
    extract::{Request, State},
    http::{HeaderMap, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::{Agent, AppState, agent_token_verification, oauth_protected_resource_metadata};

/// The token after `Bearer ` in the Authorization header.
pub fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

/// Supabase access token (C3) -> user id, `client_id`, email. `require_client_id` is set for MCP, not the phone.
pub async fn verify(
    state: &AppState,
    headers: &HeaderMap,
    require_client_id: bool,
) -> Option<agent_token_verification::Verified> {
    let token = bearer(headers)?;
    let kid = jsonwebtoken::decode_header(token).ok()?.kid?;
    let key = state.jwks.key(&kid).await?;
    agent_token_verification::check(token, &key, &state.issuer, require_client_id)
}

/// Caller of `/mcp` and `/v1/locate`: an OAuth client token, or an agent key when enabled.
pub async fn agent_user(state: &AppState, headers: &HeaderMap) -> Option<Agent> {
    if let Some(token) = verify(state, headers, true).await {
        return Some(Agent {
            user_id: token.user_id,
            client_id: token.client_id?,
            email: token.email,
            bearer: bearer(headers).map(str::to_owned),
        });
    }
    let user_id = state
        .agent_keys
        .as_ref()?
        .agent_user(bearer(headers)?)
        .await?;
    // Agent keys carry no OAuth client: the phone shows them under one stable id.
    Some(Agent {
        user_id,
        client_id: "agent-key".into(),
        email: None,
        bearer: None,
    })
}

/// MCP gate: stores the caller in the request for the tool to read.
pub async fn authorize(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(agent) = agent_user(&state, &headers).await else {
        return oauth_protected_resource_metadata::unauthorized(&state.public_url);
    };
    request.extensions_mut().insert(agent);
    next.run(request).await
}
