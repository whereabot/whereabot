use std::{sync::Arc, time::Duration};

use serde_json::Value;
use uuid::Uuid;

use super::fake_apns_and_postgrest::{Fake, start};

const EXPIRES_MS: u64 = 1_800_000_000_000;

/// (apns-push-type, apns-priority, body) per push to the live token `aa11`.
fn sent(fake: &Fake) -> Vec<(String, String, Value)> {
    let pushes = fake.pushes.lock().unwrap();
    let live = pushes.iter().filter(|(token, _, _)| token == "aa11");
    live.map(|(_, h, body)| {
        let header = |name: &str| h[name].to_str().unwrap().to_owned();
        (
            header("apns-push-type"),
            header("apns-priority"),
            body.clone(),
        )
    })
    .collect()
}

#[tokio::test]
async fn always_allowed_gets_silent_push_then_checking_alert_when_unanswered() {
    let (push, fake) = start().await;
    let id = Uuid::new_v4();
    push.wake_after(Duration::ZERO, "jwt", id, "trusted-agent", EXPIRES_MS)
        .await;
    let pushes = sent(&fake);
    assert_eq!(pushes.len(), 2);
    let (kind, priority, silent) = &pushes[0];
    assert_eq!((kind.as_str(), priority.as_str()), ("background", "5"));
    assert_eq!(silent["aps"]["content-available"], 1);
    assert!(silent["aps"].get("alert").is_none() && silent["aps"].get("sound").is_none());
    assert_eq!(silent["request_id"], id.to_string());
    assert_eq!(silent["client_id"], "trusted-agent");
    assert_eq!(silent["expires_ms"], EXPIRES_MS);
    let (kind, priority, alert) = &pushes[1];
    assert_eq!((kind.as_str(), priority.as_str()), ("alert", "10"));
    assert_eq!(alert["aps"]["category"], "LOCATION_AUTO");
    let body = alert["aps"]["alert"]["body"].as_str().unwrap();
    assert_eq!(body, "Trusted Bot is checking your location");
    assert!(!body.contains("wants to know"));
}

#[tokio::test]
async fn always_allowed_answered_after_silent_push_gets_no_alert() {
    let (push, fake) = start().await;
    let push = Arc::new(push);
    let task = tokio::spawn({
        let push = push.clone();
        async move {
            let id = Uuid::new_v4();
            push.wake_after(Duration::ZERO, "jwt", id, "trusted-agent", EXPIRES_MS)
                .await
        }
    });
    tokio::time::sleep(Duration::from_millis(150)).await;
    // The phone answered: request_location aborts the wake task.
    task.abort();
    tokio::time::sleep(Duration::from_millis(400)).await;
    let pushes = sent(&fake);
    assert_eq!(pushes.len(), 1);
    assert_eq!(pushes[0].0, "background");
}

#[tokio::test]
async fn other_agents_get_the_consent_alert_at_once() {
    let (push, fake) = start().await;
    push.wake_after(
        Duration::ZERO,
        "jwt",
        Uuid::new_v4(),
        "claude-code",
        EXPIRES_MS,
    )
    .await;
    let pushes = sent(&fake);
    assert_eq!(pushes.len(), 1);
    let (kind, _, alert) = &pushes[0];
    assert_eq!(kind, "alert");
    assert_eq!(alert["aps"]["category"], "LOCATION_REQUEST");
    assert_eq!(alert["agent_name"], "Claude Code");
    assert_eq!(
        alert["aps"]["alert"]["body"],
        "Your agent Claude Code wants to know your location"
    );
}
