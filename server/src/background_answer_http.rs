//! `POST /v1/device/answer`: the phone answers a locate request over HTTP, from a
//! notification action while the app is in the background and has no socket.

use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

use crate::{
    AppState, agent_bearer_auth, oauth_protected_resource_metadata,
    phone_message_handler::{self, Settled},
    relay_wire_protocol::FromDevice,
};

/// 204 settled, 410 gone or expired, 401 bad token, 403 agent token or another user's request.
pub async fn answer(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(message): Json<FromDevice>,
) -> Response {
    let Some(token) = agent_bearer_auth::verify(&state, &headers, false).await else {
        return oauth_protected_resource_metadata::unauthorized(&state.public_url, &headers);
    };
    // An agent token carries a client_id: an agent must never answer its own request.
    if token.client_id.is_some() {
        return StatusCode::FORBIDDEN.into_response();
    }
    let Some((id, answer)) = phone_message_handler::answer(token.user_id, message) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    // No socket answered, so no device wins: every socket gets `cancel`.
    match phone_message_handler::settle(&state, token.user_id, Uuid::nil(), id, answer).await {
        Settled::Done => StatusCode::NO_CONTENT,
        Settled::Gone => StatusCode::GONE,
        Settled::OtherUser => StatusCode::FORBIDDEN,
    }
    .into_response()
}

/// The app's webview (`tauri://localhost`, the PWA origin) posts here cross-origin.
/// Bearer auth, no cookies, so any origin is safe.
pub fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
}
