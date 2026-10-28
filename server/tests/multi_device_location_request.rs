mod common;

use common::{
    fake_phone::{DEVICE_1, DEVICE_2, connect_silent_phone, hello, next_close},
    fake_supabase::USER_A,
    locate_once, locate_settled,
    scripted_fake_phone::connect_scripted,
    start_relay,
};
use serde_json::{Value, json};

fn hello_msg(device_id: &str) -> Option<Value> {
    Some(json!({ "type": "hello", "device_id": device_id, "platform": "android" }))
}

#[tokio::test]
async fn first_answer_wins_and_the_other_device_gets_cancel() {
    let relay = start_relay().await;
    let http = reqwest::Client::new();
    let token = relay.phone(USER_A);
    let fix = |ping: &Value| {
        (ping["type"] == "locate").then(|| {
            json!({ "type": "location", "id": ping["id"], "latitude": 1.5,
                    "longitude": 2.5, "accuracy_m": 5.0, "captured_at_ms": 1 })
        })
    };
    let _a = connect_scripted(&relay.addr, &token, hello_msg(DEVICE_1), fix).await;
    let mut b = connect_scripted(&relay.addr, &token, hello_msg(DEVICE_2), |_| None).await;
    // Wait until both sockets are keyed, then ask.
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let res = locate_settled(&http, &relay.addr, &relay.mcp(USER_A)).await;
    assert_eq!(res.status(), 200);
    let ping = b.recv().await.unwrap();
    assert_eq!(ping["type"], "locate");
    let cancel = b.recv().await.unwrap();
    assert_eq!(cancel, json!({ "type": "cancel", "id": ping["id"] }));
}

#[tokio::test]
async fn same_device_id_replaces_only_itself() {
    let relay = start_relay().await;
    let token = relay.phone(USER_A);
    let mut first = connect_silent_phone(&relay.addr, &token).await;
    hello(&mut first, DEVICE_1).await;
    let mut other = connect_silent_phone(&relay.addr, &token).await;
    hello(&mut other, DEVICE_2).await;
    let mut again = connect_silent_phone(&relay.addr, &token).await;
    hello(&mut again, DEVICE_1).await;
    assert_eq!(next_close(&mut first).await, (4000, "replaced".into()));
    // The other device stays connected: a locate still reaches a socket (504, not 503).
    let http = reqwest::Client::new();
    let res = locate_once(&http, &relay.addr, &relay.mcp(USER_A)).await;
    assert_eq!(res.status(), 504);
    drop(other);
}

#[tokio::test]
async fn revoked_device_is_closed_with_4000_revoked() {
    let relay = start_relay().await;
    relay.revoked.lock().unwrap().insert(DEVICE_1.into());
    let mut ws = connect_silent_phone(&relay.addr, &relay.phone(USER_A)).await;
    hello(&mut ws, DEVICE_1).await;
    assert_eq!(next_close(&mut ws).await, (4000, "revoked".into()));
}

#[tokio::test]
async fn offline_only_when_no_device_is_connected() {
    let relay = start_relay().await;
    let http = reqwest::Client::new();
    let token = relay.mcp(USER_A);
    assert_eq!(locate_once(&http, &relay.addr, &token).await.status(), 503);
    let _silent = connect_silent_phone(&relay.addr, &relay.phone(USER_A)).await;
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    assert_eq!(locate_once(&http, &relay.addr, &token).await.status(), 504);
}
