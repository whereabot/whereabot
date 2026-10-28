mod common;

use common::{
    fake_supabase::{USER_A, USER_B},
    sharing_mcp_client::{call, phone, share},
    start_relay,
};
use serde_json::{Value, json};
#[tokio::test]
async fn shared_request_reaches_the_owners_phone_with_the_requester() {
    let relay = start_relay().await;
    relay
        .shares
        .lock()
        .unwrap()
        .push(share(USER_B, "Brother", "bro@example.com", false));
    let mut owner_pings = phone(&relay, USER_B, "location").await;
    let (error, text) = call(
        &relay,
        "get_phone_location",
        json!({ "person": "BROTHER", "timeout_secs": 5 }),
    )
    .await;
    assert!(!error, "{text}");
    assert_eq!(
        serde_json::from_str::<Value>(&text).unwrap()["latitude"],
        1.5
    );
    let ping = owner_pings.recv().await.unwrap();
    assert_eq!(ping["requester"]["user_id"], USER_A);
    assert_eq!(ping["requester"]["email"], "ofek@example.com");

    // Own request: unchanged, no requester.
    let mut own_pings = phone(&relay, USER_A, "location").await;
    let (error, _) = call(&relay, "get_phone_location", json!({ "timeout_secs": 5 })).await;
    assert!(!error);
    assert!(own_pings.recv().await.unwrap()["requester"].is_null());
}

#[tokio::test]
async fn shared_request_declined_names_the_owner() {
    let relay = start_relay().await;
    relay
        .shares
        .lock()
        .unwrap()
        .push(share(USER_B, "Brother", "bro@example.com", false));
    let _pings = phone(&relay, USER_B, "denied").await;
    let (error, text) = call(
        &relay,
        "get_phone_location",
        json!({ "person": "bro@example.com", "timeout_secs": 5 }),
    )
    .await;
    assert!(error);
    assert_eq!(text, "Brother declined to share their location.");
}
