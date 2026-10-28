use std::{sync::Arc, time::Duration};

use uuid::Uuid;

use super::fake::start;

const EXPIRES_MS: u64 = 1_800_000_000_000;

#[tokio::test]
async fn wake_pushes_each_token_once_and_drops_gone_ones() {
    let (push, fake) = start().await;
    let id = Uuid::new_v4();
    assert_eq!(push.wake("user-jwt", id, EXPIRES_MS).await, 1);
    let pushes = fake.pushes.lock().unwrap();
    assert_eq!(pushes.len(), 2);
    let (_, headers, body) = &pushes[0];
    assert_eq!(headers["apns-push-type"], "alert");
    assert_eq!(headers["apns-priority"], "10");
    assert_eq!(headers["apns-expiration"], "1800000000");
    assert_eq!(headers["apns-topic"], "io.whereabot.app");
    let auth = headers["authorization"].to_str().unwrap();
    assert!(auth.starts_with("Bearer "));
    assert_eq!(body["request_id"], id.to_string());
    assert_eq!(body["aps"]["alert"]["title"], "Whereabot");
    assert_eq!(*fake.deleted.lock().unwrap(), ["token=eq.bb22"]);
}

#[tokio::test]
async fn aborted_wake_sends_nothing_and_late_wake_sends_once() {
    let (push, fake) = start().await;
    let push = Arc::new(push);
    // The phone answers inside the delay: the caller aborts the task.
    let early = tokio::spawn({
        let push = push.clone();
        async move {
            let delay = Duration::from_millis(300);
            push.wake_after(delay, "jwt", Uuid::new_v4(), EXPIRES_MS)
                .await
        }
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    early.abort();
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert!(fake.pushes.lock().unwrap().is_empty());

    let delay = Duration::from_millis(10);
    push.wake_after(delay, "jwt", Uuid::new_v4(), EXPIRES_MS)
        .await;
    assert_eq!(fake.pushes.lock().unwrap().len(), 2);
}
