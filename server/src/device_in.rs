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
        Ok(FromDevice::Location(location)) => (location.id, Answer::Fix(location)),
        Ok(FromDevice::Denied { id, reason }) => (id, Answer::Denied(reason)),
        Ok(FromDevice::Unavailable { id, reason }) => (id, Answer::Unavailable(reason)),
        Err(_) => return,
    };
    if let Some(waiter) = state.pending.lock().await.remove(&id) {
        let _ = waiter.send(answer);
    }
}
