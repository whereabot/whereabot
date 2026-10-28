mod common;

use common::{
    locate_settled,
    phone::{DEVICE_1, connect_silent_phone, hello, next_close},
    scripted::connect_scripted,
    start_relay,
    supabase::USER_A,
};
use serde_json::{Value, json};

#[tokio::test]
async fn same_device_id_closes_the_old_socket_with_4000() {
    let relay = start_relay().await;
    let mut old = connect_silent_phone(&relay.addr, &relay.phone(USER_A)).await;
    hello(&mut old, DEVICE_1).await;
    let mut new = connect_silent_phone(&relay.addr, &relay.phone(USER_A)).await;
    hello(&mut new, DEVICE_1).await;
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
