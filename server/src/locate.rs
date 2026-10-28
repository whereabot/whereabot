use std::{sync::Arc, time::Duration};

use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::{
    Agent, AppState, auth,
    protocol::{Answer, Location, ToDevice},
};

#[derive(Deserialize)]
pub struct LocateParams {
    timeout_secs: Option<u64>,
}

pub enum LocateError {
    Offline,
    Timeout,
    Denied,
}

/// Pings one user's phone and waits for its fix.
pub async fn request_location(
    state: &AppState,
    agent: &Agent,
    timeout: Duration,
) -> Result<Location, LocateError> {
    // iOS drops the socket in the background: wait for the app, and resend the
    // ping to each new socket so a reconnect does not lose it.
    let deadline = tokio::time::Instant::now() + timeout;
    let id = Uuid::new_v4();
    let (tx, mut rx) = oneshot::channel();
    state.pending.lock().await.insert(id, tx);
    let mut sent_to: Option<mpsc::Sender<ToDevice>> = None;
    let result = loop {
        let current = state.devices.lock().await.get(&agent.user_id).cloned();
        if let Some(device) = current
            && !sent_to.as_ref().is_some_and(|s| s.same_channel(&device))
            && device
                .send(ToDevice::Locate {
                    id,
                    client_id: agent.client_id.clone(),
                })
                .await
                .is_ok()
        {
            sent_to = Some(device);
        }
        if tokio::time::Instant::now() >= deadline {
            break Err(if sent_to.is_some() {
                LocateError::Timeout
            } else {
                LocateError::Offline
            });
        }
        if let Ok(fix) = tokio::time::timeout(Duration::from_millis(250), &mut rx).await {
            break match fix {
                Ok(Answer::Fix(location)) => Ok(location),
                Ok(Answer::Denied) => Err(LocateError::Denied),
                Err(_) => Err(LocateError::Timeout),
            };
        }
    };
    state.pending.lock().await.remove(&id);
    result
}

pub async fn locate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<LocateParams>,
) -> Response {
    let Some(agent) = auth::agent_user(&state, &headers).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let timeout = Duration::from_secs(params.timeout_secs.unwrap_or(30).min(120));
    match request_location(&state, &agent, timeout).await {
        Ok(location) => Json(location).into_response(),
        Err(LocateError::Offline) => {
            (StatusCode::SERVICE_UNAVAILABLE, "device offline").into_response()
        }
        Err(LocateError::Denied) => (StatusCode::FORBIDDEN, "user declined").into_response(),
        Err(LocateError::Timeout) => {
            (StatusCode::GATEWAY_TIMEOUT, "device did not answer").into_response()
        }
    }
}
