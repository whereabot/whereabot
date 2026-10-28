mod common;

use common::{
    locate_settled, phone::connect_silent_phone, scripted::connect_scripted, start_relay,
    supabase::USER_A,
};
use futures_util::StreamExt;
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::Message;

/// The close code and reason of the next Close frame, within 5 s.
async fn next_close(
    ws: &mut (impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin),
) -> (u16, String) {
    let wait = async {
        while let Some(Ok(msg)) = ws.next().await {
            if let Message::Close(Some(frame)) = msg {
                return (u16::from(frame.code), frame.reason.to_string());
            }
        }
        panic!("socket ended without a Close frame");
    };
    tokio::time::timeout(std::time::Duration::from_secs(5), wait)
        .await
        .expect("no Close frame")
}

#[tokio::test]
async fn new_phone_closes_the_old_socket_with_4000() {
    let relay = start_relay().await;
    let mut old = connect_silent_phone(&relay.addr, &relay.phone(USER_A)).await;
    let _new = connect_silent_phone(&relay.addr, &relay.phone(USER_A)).await;
    assert_eq!(next_close(&mut old).await, (4000, "replaced".into()));
}

#[tokio::test]
async fn shutdown_closes_every_device_with_1012() {
    let relay = start_relay().await;
    let mut a = connect_silent_phone(&relay.addr, &relay.phone(USER_A)).await;
    let mut b = connect_silent_phone(&relay.addr, &relay.phone(common::supabase::USER_B)).await;
    relay.shutdown.close_devices().await;
    assert_eq!(next_close(&mut a).await, (1012, "restart".into()));
    assert_eq!(next_close(&mut b).await, (1012, "restart".into()));
}

#[tokio::test]
async fn ping_carries_expires_ms_and_hello_is_accepted() {
    let relay = start_relay().await;
    let http = reqwest::Client::new();
    let hello = Some(json!({ "type": "hello", "app_version": "0.1.6" }));
    let reply = |ping: &Value| Some(json!({ "type": "denied", "id": ping["id"] }));
    let mut seen = connect_scripted(&relay.addr, &relay.phone(USER_A), hello, reply).await;
    let before = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    // timeout_secs=5 in the helper: the relay stops waiting about 5 s from now.
    locate_settled(&http, &relay.addr, &relay.mcp(USER_A)).await;
    let ping = seen.recv().await.unwrap();
    let expires = ping["expires_ms"].as_u64().unwrap();
    assert!(
        expires > before + 4_000 && expires < before + 7_000,
        "{expires} vs {before}"
    );
}
