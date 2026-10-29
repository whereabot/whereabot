use serde_json::{Value, json};
use uuid::Uuid;
use web_push_native::{Auth, decrypt};

use super::{
    fake_web_push_postgrest::{AUTH, GONE_ID},
    fake_web_push_service::{FakeWeb, start},
};
use crate::{
    push::{apns_sender::Kind, fake_share_rpcs::OWNER, push_wake_tests::alert},
    relay_wire_protocol::Requester,
};

/// The JSON `sw.js` gets for push `i`.
fn shown(fake: &FakeWeb, i: usize) -> Value {
    let body = fake.pushes.lock().unwrap()[i].2.to_vec();
    let plain = decrypt(body, &fake.ua, &Auth::clone_from_slice(&AUTH)).unwrap();
    serde_json::from_slice(&plain).unwrap()
}

#[tokio::test]
async fn alert_goes_to_each_subscription_encrypted_and_gone_ones_are_dropped() {
    let (push, fake) = start().await;
    let id = Uuid::new_v4();
    assert_eq!(push.wake("jwt", &alert(id, "claude-code")).await, 1);
    {
        let pushes = fake.pushes.lock().unwrap();
        assert_eq!(pushes.len(), 2);
        let (name, h, _) = &pushes[0];
        assert_eq!(name, "live");
        assert_eq!(h["content-encoding"], "aes128gcm");
        assert_eq!(h["urgency"], "high");
        assert!(h["authorization"].to_str().unwrap().starts_with("vapid t="));
        assert!(h["ttl"].to_str().unwrap().parse::<u64>().unwrap() >= 60);
    }
    let body = shown(&fake, 0);
    assert_eq!(body["title"], "Whereabot");
    assert_eq!(
        body["body"],
        "Your agent Claude Code wants to know your location"
    );
    assert_eq!(body["request_id"], id.to_string());
    assert_eq!(*fake.deleted.lock().unwrap(), [format!("id=eq.{GONE_ID}")]);
}

#[tokio::test]
async fn shared_request_reaches_only_the_owner_and_names_the_requester() {
    let (push, fake) = start().await;
    let requester = Requester {
        user_id: Uuid::new_v4(),
        name: "Ofek".into(),
        email: None,
    };
    let mut shared = alert(Uuid::new_v4(), "claude-code");
    (shared.owner, shared.requester) = (Some(OWNER.parse().unwrap()), Some(&requester));
    assert_eq!(push.wake("requester-jwt", &shared).await, 1);
    assert_eq!(fake.pushes.lock().unwrap()[0].0, "owner");
    let text = "Ofek's agent Claude Code wants to know your location";
    assert_eq!(shown(&fake, 0)["body"], text);
    assert!(fake.deleted.lock().unwrap().is_empty());
}

#[tokio::test]
async fn silent_wake_ups_never_go_to_browsers() {
    let (push, fake) = start().await;
    let a = alert(Uuid::new_v4(), "c");
    assert_eq!(
        push.send_all("jwt", &a, &json!({}), Kind::Background).await,
        0
    );
    assert!(fake.pushes.lock().unwrap().is_empty());
}
