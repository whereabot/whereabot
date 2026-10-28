mod common;

use common::{
    Relay,
    fake_supabase::{USER_A, USER_B},
    locate_settled,
    scripted_fake_phone::connect_scripted,
    start_relay,
};
use serde_json::{Value, json};
use tokio::task::JoinHandle;

/// Starts a locate for USER_A and returns it with the request id a silent phone saw.
async fn pending_request(relay: &Relay) -> (JoinHandle<reqwest::Response>, String) {
    let mut pings = connect_scripted(&relay.addr, &relay.phone(USER_A), None, |_| None).await;
    let (addr, token) = (relay.addr.clone(), relay.mcp(USER_A));
    let locate =
        tokio::spawn(async move { locate_settled(&reqwest::Client::new(), &addr, &token).await });
    let ping = pings.recv().await.unwrap();
    (locate, ping["id"].as_str().unwrap().to_owned())
}

async fn post_answer(relay: &Relay, token: &str, body: Value) -> u16 {
    reqwest::Client::new()
        .post(format!("http://{}/v1/device/answer", relay.addr))
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .unwrap()
        .status()
        .as_u16()
}

#[tokio::test]
async fn fix_over_http_settles_the_request() {
    let relay = start_relay().await;
    let (locate, id) = pending_request(&relay).await;
    let fix = json!({ "type": "location", "id": id, "latitude": 32.1, "longitude": 34.8,
                      "accuracy_m": 12.0, "captured_at_ms": 7 });
    assert_eq!(post_answer(&relay, &relay.phone(USER_A), fix).await, 204);
    let res = locate.await.unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.json::<Value>().await.unwrap()["latitude"], 32.1);
}

#[tokio::test]
async fn denied_over_http_returns_403_to_the_agent() {
    let relay = start_relay().await;
    let (locate, id) = pending_request(&relay).await;
    let denied = json!({ "type": "denied", "id": id, "reason": "user" });
    assert_eq!(post_answer(&relay, &relay.phone(USER_A), denied).await, 204);
    assert_eq!(locate.await.unwrap().status(), 403);
}

#[tokio::test]
async fn unknown_or_expired_id_is_410() {
    let relay = start_relay().await;
    let denied = json!({ "type": "denied", "id": uuid::Uuid::new_v4(), "reason": "user" });
    assert_eq!(post_answer(&relay, &relay.phone(USER_A), denied).await, 410);
}

#[tokio::test]
async fn other_user_or_agent_token_cannot_answer() {
    let relay = start_relay().await;
    let (locate, id) = pending_request(&relay).await;
    let denied = json!({ "type": "denied", "id": id, "reason": "user" });
    assert_eq!(
        post_answer(&relay, &relay.phone(USER_B), denied.clone()).await,
        403
    );
    assert_eq!(
        post_answer(&relay, &relay.mcp(USER_A), denied.clone()).await,
        403
    );
    assert_eq!(post_answer(&relay, "not-a-jwt", denied.clone()).await, 401);
    // The owner can still answer: the failed tries did not consume the request.
    assert_eq!(post_answer(&relay, &relay.phone(USER_A), denied).await, 204);
    assert_eq!(locate.await.unwrap().status(), 403);
}
