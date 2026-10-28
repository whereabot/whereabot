use std::time::Duration;

use serde_json::Value;
use uuid::Uuid;

use super::{
    fake_apns_and_postgrest::{Fake, start},
    fake_share_rpcs::OWNER,
    push_wake_tests::alert,
};
use crate::relay_wire_protocol::Requester;

fn ofek() -> Requester {
    Requester {
        user_id: Uuid::new_v4(),
        name: "Ofek".into(),
        email: Some("ofek@example.com".into()),
    }
}

/// (token, apns-push-type, body) per push.
fn sent(fake: &Fake) -> Vec<(String, String, Value)> {
    let pushes = fake.pushes.lock().unwrap();
    let kind = |h: &axum::http::HeaderMap| h["apns-push-type"].to_str().unwrap().to_owned();
    pushes
        .iter()
        .map(|(t, h, b)| (t.clone(), kind(h), b.clone()))
        .collect()
}

#[tokio::test]
async fn shared_request_alerts_the_owners_phone_and_names_the_requester() {
    let (push, fake) = start().await;
    let requester = ofek();
    let mut shared = alert(Uuid::new_v4(), "claude-code");
    (shared.owner, shared.requester) = (Some(OWNER.parse().unwrap()), Some(&requester));
    push.wake_after(Duration::ZERO, "requester-jwt", &shared)
        .await;
    let pushes = sent(&fake);
    // Only the owner's token: never the requester's own `aa11`/`bb22`.
    assert_eq!(pushes.len(), 1);
    let (token, kind, body) = &pushes[0];
    assert_eq!((token.as_str(), kind.as_str()), ("cc33", "alert"));
    assert_eq!(
        body["aps"]["alert"]["body"],
        "Ofek's agent Claude Code wants to know your location"
    );
    assert_eq!(body["aps"]["category"], "LOCATION_REQUEST");
    assert_eq!(body["requester"]["name"], "Ofek");
    assert_eq!(body["requester"]["user_id"], requester.user_id.to_string());
}

#[tokio::test]
async fn always_allowed_for_requester_gets_silent_push_then_checking() {
    let (push, fake) = start().await;
    let requester = ofek();
    let mut shared = alert(Uuid::new_v4(), "trusted-agent");
    (shared.owner, shared.requester) = (Some(OWNER.parse().unwrap()), Some(&requester));
    push.wake_after(Duration::ZERO, "requester-jwt", &shared)
        .await;
    let pushes = sent(&fake);
    assert_eq!(pushes.len(), 2);
    assert_eq!(
        (pushes[0].0.as_str(), pushes[0].1.as_str()),
        ("cc33", "background")
    );
    assert_eq!(pushes[0].2["requester"]["name"], "Ofek");
    let body = &pushes[1].2;
    assert_eq!(body["aps"]["category"], "LOCATION_AUTO");
    assert_eq!(
        body["aps"]["alert"]["body"],
        "Ofek's agent Trusted Bot is checking your location"
    );
}

#[tokio::test]
async fn no_active_share_means_no_push_and_no_token_delete() {
    let (push, fake) = start().await;
    let requester = ofek();
    let mut shared = alert(Uuid::new_v4(), "trusted-agent");
    // Another owner: the RPCs find no share, so no tokens and no always-allow.
    (shared.owner, shared.requester) = (Some(Uuid::new_v4()), Some(&requester));
    push.wake_after(Duration::ZERO, "requester-jwt", &shared)
        .await;
    assert!(sent(&fake).is_empty());
    assert!(fake.deleted.lock().unwrap().is_empty());
}
