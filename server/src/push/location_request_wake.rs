use std::{sync::Arc, time::Duration};

use tokio::task::JoinHandle;
use uuid::Uuid;

use super::{Push, location_request_push_payload::Alert};
use crate::relay_wire_protocol::Requester;

/// A connected phone gets this long to answer before the push goes out.
const GRACE: Duration = Duration::from_secs(2);

/// One locate request, owned so the wake task can keep it.
pub struct Request {
    pub id: Uuid,
    pub client_id: String,
    pub expires_ms: u64,
    /// The owner and the requester when someone else's agent asks.
    pub shared: Option<(Uuid, Requester)>,
}

/// The running wake-up push. Dropping it aborts the push, so a request that ends,
/// or whose caller hangs up, sends nothing more.
pub struct WakeTask(JoinHandle<()>);

impl Drop for WakeTask {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Starts the wake-up push for one request. Keep the task while the request is open.
/// None when push is off or the caller has no user JWT (agent keys).
pub fn start(
    push: Option<&Arc<Push>>,
    bearer: Option<&str>,
    phone_connected: bool,
    request: Request,
) -> Option<WakeTask> {
    let (push, bearer) = (Arc::clone(push?), bearer?.to_owned());
    let delay = if phone_connected {
        GRACE
    } else {
        Duration::ZERO
    };
    Some(WakeTask(tokio::spawn(async move {
        let alert = Alert {
            request_id: request.id,
            client_id: &request.client_id,
            expires_ms: request.expires_ms,
            owner: request.shared.as_ref().map(|(owner, _)| *owner),
            requester: request.shared.as_ref().map(|(_, requester)| requester),
        };
        push.wake_after(delay, &bearer, &alert).await;
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> Request {
        Request {
            id: Uuid::new_v4(),
            client_id: "claude-code".into(),
            expires_ms: 1_800_000_000_000,
            shared: None,
        }
    }

    #[tokio::test]
    async fn pushes_at_once_when_offline_and_never_without_user_jwt() {
        let (push, fake) = super::super::fake_apns_and_postgrest::start().await;
        let push = Arc::new(push);
        assert!(start(None, Some("jwt"), false, request()).is_none());
        assert!(start(Some(&push), None, false, request()).is_none());
        let mut task = start(Some(&push), Some("jwt"), false, request()).unwrap();
        (&mut task.0).await.unwrap();
        assert!(!fake.pushes.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn dropping_the_task_during_the_grace_sends_no_push() {
        let (push, fake) = super::super::fake_apns_and_postgrest::start().await;
        let push = Arc::new(push);
        // Phone connected: the push waits GRACE. The caller hangs up before that.
        drop(start(Some(&push), Some("jwt"), true, request()));
        tokio::time::sleep(GRACE + Duration::from_millis(500)).await;
        assert!(fake.pushes.lock().unwrap().is_empty());
    }
}
