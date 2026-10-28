use std::{sync::Arc, time::Duration};

use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::{
    AppState, auth,
    protocol::{Location, ToDevice},
};

#[derive(Deserialize)]
pub struct LocateParams {
    timeout_secs: Option<u64>,
}

pub enum LocateError {
    Offline,
    Timeout,
}

/// Pings one user's phone and waits for its fix.
pub async fn request_location(
    state: &AppState,
    user_id: Uuid,
    timeout: Duration,
) -> Result<Location, LocateError> {
    // iOS drops the socket in the background; wait for the user to open the app.
    let deadline = tokio::time::Instant::now() + timeout;
    let device = loop {
        if let Some(d) = state.devices.lock().await.get(&user_id).cloned() {
            break d;
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(LocateError::Offline);
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    };
    let id = Uuid::new_v4();
    let (tx, rx) = oneshot::channel();
    state.pending.lock().await.insert(id, tx);
    if device.send(ToDevice::Locate { id }).await.is_err() {
        state.pending.lock().await.remove(&id);
        return Err(LocateError::Offline);
    }
    match tokio::time::timeout_at(deadline, rx).await {
        Ok(Ok(location)) => Ok(location),
        _ => {
            state.pending.lock().await.remove(&id);
            Err(LocateError::Timeout)
        }
    }
}

pub async fn locate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<LocateParams>,
) -> Response {
    let Some(user_id) = auth::agent_user(&state, &headers).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let timeout = Duration::from_secs(params.timeout_secs.unwrap_or(30).min(120));
    match request_location(&state, user_id, timeout).await {
        Ok(location) => Json(location).into_response(),
        Err(LocateError::Offline) => {
            (StatusCode::SERVICE_UNAVAILABLE, "device offline").into_response()
        }
        Err(LocateError::Timeout) => {
            (StatusCode::GATEWAY_TIMEOUT, "device did not answer").into_response()
        }
    }
}
