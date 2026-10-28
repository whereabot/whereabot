use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

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
    errors::LocateError,
    protocol::{Answer, Location, ToDevice},
};

#[derive(Deserialize)]
pub struct LocateParams {
    timeout_secs: Option<u64>,
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
    let expires_ms = (SystemTime::now() + timeout)
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_millis() as u64);
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
                    expires_ms,
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
                LocateError::Offline {
                    email: agent.email.clone(),
                }
            });
        }
        if let Ok(fix) = tokio::time::timeout(Duration::from_millis(250), &mut rx).await {
            break match fix {
                Ok(Answer::Fix(location)) => Ok(location),
                Ok(Answer::Denied(reason)) => Err(LocateError::Denied(reason)),
                Ok(Answer::Unavailable(reason)) => Err(LocateError::Unavailable(reason)),
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
    let timeout = Duration::from_secs(params.timeout_secs.unwrap_or(60).min(120));
    match request_location(&state, &agent, timeout).await {
        Ok(location) => Json(location).into_response(),
        Err(error) => (error.status(), error.message()).into_response(),
    }
}
