//! `POST /v1/locate`: the HTTP twin of the MCP tool, for agents without MCP.

use std::{sync::Arc, time::Duration};

use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header::RETRY_AFTER},
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::{AppState, agent_bearer_auth, location_target::locate_person};

#[derive(Deserialize)]
pub struct LocateParams {
    timeout_secs: Option<u64>,
    /// Email or display name of someone who shares with the caller; absent = own phones.
    person: Option<String>,
}

pub async fn locate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<LocateParams>,
) -> Response {
    let Some(agent) = agent_bearer_auth::agent_user(&state, &headers).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let timeout = Duration::from_secs(params.timeout_secs.unwrap_or(60).min(120));
    match locate_person(&state, &agent, params.person.as_deref(), timeout).await {
        Ok(location) => Json(location).into_response(),
        Err(error) => {
            let retry = error
                .retry_after_secs()
                .map(|secs| [(RETRY_AFTER, secs.to_string())]);
            (error.status(), retry, error.message()).into_response()
        }
    }
}
