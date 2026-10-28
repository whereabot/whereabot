use std::{sync::Arc, time::Duration};

use uuid::Uuid;

use super::{fake_apns_and_postgrest::start, location_request_push_payload::Alert};

pub const EXPIRES_MS: u64 = 1_800_000_000_000;

pub fn alert(request_id: Uuid, client_id: &str) -> Alert<'_> {
    Alert {
        request_id,
        client_id,
        expires_ms: EXPIRES_MS,
        owner: None,
        requester: None,
    }
}

#[tokio::test]
async fn wake_pushes_each_token_once_and_drops_gone_ones() {
    let (push, fake) = start().await;
    let id = Uuid::new_v4();
    assert_eq!(push.wake("user-jwt", &alert(id, "claude-code")).await, 1);
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
async fn push_is_actionable_and_names_the_agent() {
    let (push, fake) = start().await;
    let id = Uuid::new_v4();
    push.wake("jwt", &alert(id, "claude-code")).await;
    push.wake("jwt", &alert(id, "unknown-client")).await;
    let pushes = fake.pushes.lock().unwrap();
    let (named, unnamed) = (&pushes[0].2, &pushes[2].2);
    assert_eq!(named["aps"]["category"], "LOCATION_REQUEST");
    assert_eq!(named["client_id"], "claude-code");
    assert_eq!(named["expires_ms"], EXPIRES_MS);
    assert_eq!(
        named["aps"]["alert"]["body"],
        "Your agent Claude Code wants to know your location"
    );
    assert_eq!(
        unnamed["aps"]["alert"]["body"],
        "An agent wants to know your location"
    );
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
            push.wake_after(delay, "jwt", &alert(Uuid::new_v4(), "c"))
                .await
        }
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    early.abort();
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert!(fake.pushes.lock().unwrap().is_empty());

    let delay = Duration::from_millis(10);
    push.wake_after(delay, "jwt", &alert(Uuid::new_v4(), "c"))
        .await;
    assert_eq!(fake.pushes.lock().unwrap().len(), 2);
}
