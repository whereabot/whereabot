use std::{sync::Arc, time::Duration};

use tokio::task::JoinHandle;
use uuid::Uuid;

use super::Push;

/// A connected phone gets this long to answer before the push goes out.
const GRACE: Duration = Duration::from_secs(2);

/// Starts the wake-up push for one request. Abort the handle when the request ends.
/// None when push is off or the caller has no user JWT (agent keys).
pub fn start(
    push: Option<&Arc<Push>>,
    bearer: Option<&str>,
    phone_connected: bool,
    request_id: Uuid,
    expires_ms: u64,
) -> Option<JoinHandle<()>> {
    let (push, bearer) = (Arc::clone(push?), bearer?.to_owned());
    let delay = if phone_connected {
        GRACE
    } else {
        Duration::ZERO
    };
    Some(tokio::spawn(async move {
        push.wake_after(delay, &bearer, request_id, expires_ms)
            .await;
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn pushes_at_once_when_offline_and_never_without_user_jwt() {
        let (push, fake) = super::super::fake_apns_and_postgrest::start().await;
        let push = Arc::new(push);
        assert!(start(None, Some("jwt"), false, Uuid::new_v4(), 0).is_none());
        assert!(start(Some(&push), None, false, Uuid::new_v4(), 0).is_none());
        let task = start(
            Some(&push),
            Some("jwt"),
            false,
            Uuid::new_v4(),
            1_800_000_000_000,
        );
        task.unwrap().await.unwrap();
        assert!(!fake.pushes.lock().unwrap().is_empty());
    }
}
