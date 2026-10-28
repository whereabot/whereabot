use uuid::Uuid;

use crate::{
    AppState,
    protocol::{Answer, FromDevice},
};

/// Handles one text message from a phone. Unknown or malformed messages are ignored.
pub async fn handle(state: &AppState, user_id: Uuid, text: &str) {
    let (id, answer) = match serde_json::from_str(text) {
        Ok(FromDevice::Hello { app_version }) => {
            tracing::info!(%user_id, ?app_version, "device hello");
            return;
        }
        Ok(FromDevice::Log { msg }) => {
            let msg: String = msg.chars().take(200).collect();
            tracing::info!(%user_id, %msg, "device log");
            return;
        }
        Ok(FromDevice::Location(location)) => {
            tracing::info!(%user_id, id = %location.id, "device answered with a fix");
            (location.id, Answer::Fix(location))
        }
        Ok(FromDevice::Denied { id, reason }) => {
            tracing::info!(%user_id, %id, ?reason, "device denied");
            (id, Answer::Denied(reason))
        }
        Ok(FromDevice::Unavailable { id, reason }) => {
            tracing::info!(%user_id, %id, ?reason, "device unavailable");
            (id, Answer::Unavailable(reason))
        }
        Err(e) => {
            tracing::warn!(%user_id, error = %e, "device sent an unknown message");
            return;
        }
    };
    if let Some(waiter) = state.pending.lock().await.remove(&id) {
        let _ = waiter.send(answer);
    }
}
