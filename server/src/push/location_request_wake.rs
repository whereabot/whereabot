use std::{sync::Arc, time::Duration};

use tokio::task::JoinHandle;
use uuid::Uuid;

use super::{Push, location_request_push_payload::Alert};
use crate::locate_request::LocateRequest;

/// A connected phone gets this long to answer before the push goes out.
const GRACE: Duration = Duration::from_secs(2);

/// The running wake-up push. Dropping it aborts the push, so a request that ends,
/// or whose caller hangs up, sends nothing more.
pub struct WakeTask(JoinHandle<()>);

impl Drop for WakeTask {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Starts the wake-up push for one request. Keep the task while the request is open.
/// `owner` is set only when someone else's agent asks. None when push is off or the
/// caller has no user JWT (agent keys).
pub fn start(
    push: Option<&Arc<Push>>,
    bearer: Option<&str>,
    phone_connected: bool,
    request: Arc<LocateRequest>,
    owner: Option<Uuid>,
) -> Option<WakeTask> {
    let (push, bearer) = (Arc::clone(push?), bearer?.to_owned());
    let delay = if phone_connected {
        GRACE
    } else {
        Duration::ZERO
    };
    Some(WakeTask(tokio::spawn(async move {
        let alert = Alert {
            request: &request,
            owner,
        };
        push.wake_after(delay, &bearer, &alert).await;
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> Arc<LocateRequest> {
        Arc::new(LocateRequest {
            id: Uuid::new_v4(),
            client_id: "claude-code".into(),
            expires_ms: 1_800_000_000_000,
            requester: None,
            reason: None,
        })
    }

    #[tokio::test]
    async fn pushes_at_once_when_offline_and_never_without_user_jwt() {
        let (push, fake) = super::super::fake_apns_and_postgrest::start().await;
        let push = Arc::new(push);
        assert!(start(None, Some("jwt"), false, request(), None).is_none());
        assert!(start(Some(&push), None, false, request(), None).is_none());
        let mut task = start(Some(&push), Some("jwt"), false, request(), None).unwrap();
        (&mut task.0).await.unwrap();
        assert!(!fake.pushes.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn dropping_the_task_during_the_grace_sends_no_push() {
        let (push, fake) = super::super::fake_apns_and_postgrest::start().await;
        let push = Arc::new(push);
        // Phone connected: the push waits GRACE. The caller hangs up before that.
        drop(start(Some(&push), Some("jwt"), true, request(), None));
        tokio::time::sleep(GRACE + Duration::from_millis(500)).await;
        assert!(fake.pushes.lock().unwrap().is_empty());
    }
}
