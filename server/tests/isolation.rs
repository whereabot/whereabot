mod common;

use common::{
    locate_once, locate_settled,
    mcp::{OFFLINE, call_location, call_settled, rpc},
    phone::connect_phone,
    start_relay,
    supabase::{USER_A, USER_B},
};
use serde_json::Value;

#[tokio::test]
async fn user_b_token_never_reaches_phone_a() {
    let relay = start_relay().await;
    let http = reqwest::Client::new();
    let (token_a, token_b) = (relay.mcp(USER_A), relay.mcp(USER_B));
    connect_phone(&relay.addr, &relay.phone(USER_A), 32.08).await;
    let a = locate_settled(&http, &relay.addr, &token_a).await;
    assert_eq!(a.status(), 200);

    // Only phone A is connected. User B must get no fix from it.
    assert_eq!(
        locate_once(&http, &relay.addr, &token_b).await.status(),
        503
    );

    connect_phone(&relay.addr, &relay.phone(USER_B), 51.5).await;
    let b: Value = locate_settled(&http, &relay.addr, &token_b)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(b["latitude"], 51.5);
    let a: Value = locate_once(&http, &relay.addr, &token_a)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(a["latitude"], 32.08);
}

#[tokio::test]
async fn mcp_user_b_token_never_reaches_phone_a() {
    let relay = start_relay().await;
    let http = reqwest::Client::new();
    connect_phone(&relay.addr, &relay.phone(USER_A), 32.08).await;
    let a = call_settled(&http, &relay.addr, &relay.mcp(USER_A), 3).await;
    assert!(a["result"]["isError"] != true);

    let b = rpc(&http, &relay.addr, &relay.mcp(USER_B), call_location(4)).await;
    assert_eq!(b["result"]["isError"], true);
    assert_eq!(b["result"]["content"][0]["text"], OFFLINE);
}
