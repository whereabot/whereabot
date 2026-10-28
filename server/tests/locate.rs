mod common;

use common::{locate_once, locate_settled, phone::connect_phone, start_relay, supabase::USER_A};
use serde_json::Value;

#[tokio::test]
async fn locate_rejects_bad_token() {
    let relay = start_relay().await;
    let http = reqwest::Client::new();
    assert_eq!(locate_once(&http, &relay.addr, "wrong").await.status(), 401);
    let stray = common::jwt::sign_with_kid(
        &common::jwt::claims(&relay.issuer, USER_A, true),
        "other-kid",
    );
    assert_eq!(locate_once(&http, &relay.addr, &stray).await.status(), 401);
}

#[tokio::test]
async fn locate_returns_offline_then_phone_fix() {
    let relay = start_relay().await;
    let http = reqwest::Client::new();
    let token = relay.mcp(USER_A);
    assert_eq!(locate_once(&http, &relay.addr, &token).await.status(), 503);

    connect_phone(&relay.addr, &relay.phone(USER_A), 32.08).await;
    let res = locate_settled(&http, &relay.addr, &token).await;
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["latitude"], 32.08);
    assert_eq!(body["longitude"], 34.78);
}
