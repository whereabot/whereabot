use tokio::sync::oneshot;
use uuid::Uuid;

use crate::{
    AppState, pending_location_requests,
    relay_wire_protocol::{Answer, FromDevice},
};

/// A waiting request: the asked user, and where its answer goes with the answering device.
pub type Waiter = (Uuid, oneshot::Sender<(Uuid, Answer)>);

/// What became of one answer.
pub enum Settled {
    Done,
    /// No request waits under this id: it ended, timed out, or never existed.
    Gone,
    /// The request belongs to another user.
    OtherUser,
}

/// Handles one text message from a phone. Unknown or malformed messages are ignored.
/// `device` is the socket's registry key. Returns the `device_id` of a `hello`.
pub async fn handle(state: &AppState, user_id: Uuid, device: Uuid, text: &str) -> Option<Uuid> {
    match serde_json::from_str(text) {
        Ok(FromDevice::Hello {
            app_version,
            device_id,
            device_name,
            platform,
        }) => {
            tracing::info!(%user_id, ?app_version, ?device_id, ?device_name, ?platform, "device hello");
            return device_id;
        }
        Ok(FromDevice::Log { msg }) => {
            let msg: String = msg.chars().take(200).collect();
            tracing::info!(%user_id, %msg, "device log");
        }
        Ok(message) => {
            if let Some((id, answer)) = answer(user_id, message) {
                settle(state, user_id, device, id, answer).await;
            }
        }
        Err(e) => tracing::warn!(%user_id, error = %e, "device sent an unknown message"),
    }
    None
}

/// The answer a message carries, logged. None for `hello` and `log`.
pub fn answer(user_id: Uuid, message: FromDevice) -> Option<(Uuid, Answer)> {
    match message {
        FromDevice::Location(location) => {
            tracing::info!(%user_id, id = %location.id, "device answered with a fix");
            Some((location.id, Answer::Fix(location)))
        }
        FromDevice::Denied { id, reason } => {
            tracing::info!(%user_id, %id, ?reason, "device denied");
            Some((id, Answer::Denied(reason)))
        }
        FromDevice::Unavailable { id, reason } => {
            tracing::info!(%user_id, %id, ?reason, "device unavailable");
            Some((id, Answer::Unavailable(reason)))
        }
        FromDevice::Hello { .. } | FromDevice::Log { .. } => None,
    }
}

/// Hands the answer to the waiting request when `user_id` owns it.
pub async fn settle(
    state: &AppState,
    user_id: Uuid,
    device: Uuid,
    id: Uuid,
    answer: Answer,
) -> Settled {
    let mut pending = pending_location_requests::lock(state);
    match pending.waiters.get(&id) {
        None => return Settled::Gone,
        Some((owner, _)) if *owner != user_id => return Settled::OtherUser,
        Some(_) => {}
    }
    let Some((_, waiter)) = pending.waiters.remove(&id) else {
        return Settled::Gone;
    };
    match waiter.send((device, answer)) {
        Ok(()) => Settled::Done,
        Err(_) => Settled::Gone,
    }
}
