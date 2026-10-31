//! What the agent hears when the phone got the ping but no answer came back in time.

mod common;

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use common::{
    fake_phone::connect_silent_phone, fake_supabase::USER_A, locate_once,
    scripted_fake_phone::connect_scripted, sharing_mcp_client::call, start_relay,
};
use futures_util::StreamExt;
use serde_json::{Value, json};

const NO_ANSWER: &str =
    "The phone did not answer in time. Ask the user to open Whereabot and retry.";
const CONNECTION_LOST: &str = "The phone got the request, then lost its connection before it could answer (weak or no network). Ask the user to check the phone's network, then retry.";

#[tokio::test]
async fn silent_but_connected_phone_is_no_answer() {
    let relay = start_relay().await;
    let _phone = connect_silent_phone(&relay.addr, &relay.phone(USER_A)).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let res = locate_once(&reqwest::Client::new(), &relay.addr, &relay.mcp(USER_A)).await;
    assert_eq!(res.status(), 504);
    assert_eq!(res.text().await.unwrap(), NO_ANSWER);
}

#[tokio::test]
async fn phone_that_drops_after_the_ping_is_connection_lost() {
    let relay = start_relay().await;
    let mut phone = connect_silent_phone(&relay.addr, &relay.phone(USER_A)).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let pending = tokio::spawn({
        let (addr, token) = (relay.addr.clone(), relay.mcp(USER_A));
        async move { locate_once(&reqwest::Client::new(), &addr, &token).await }
    });
    // The ping reached the phone; then its network dies (the 2026-10-31 ship Wi-Fi case).
    while let Some(Ok(msg)) = phone.next().await {
        if msg.is_text() {
            break;
        }
    }
    drop(phone);
    let res = pending.await.unwrap();
    assert_eq!(res.status(), 504);
    assert_eq!(res.text().await.unwrap(), CONNECTION_LOST);
}

#[tokio::test]
async fn mcp_wait_ends_before_the_clients_60_s_abort() {
    let relay = start_relay().await;
    let deny = |ping: &Value| Some(json!({ "type": "denied", "id": ping["id"] }));
    let mut pings = connect_scripted(&relay.addr, &relay.phone(USER_A), None, deny).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    call(&relay, "get_phone_location", json!({ "timeout_secs": 120 })).await;
    let expires = pings.recv().await.unwrap()["expires_ms"].as_u64().unwrap();
    assert!(
        expires <= now_ms + 56_000,
        "relay waits past 55 s: {expires} vs {now_ms}"
    );
}
