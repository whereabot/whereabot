mod common;

use common::{
    locate_once, locate_settled,
    phone::connect_phone,
    start_relay,
    supabase::{agent_key_a, unknown_key},
};
use serde_json::Value;

#[tokio::test]
async fn locate_rejects_unknown_key() {
    let addr = start_relay().await;
    let http = reqwest::Client::new();
    assert_eq!(locate_once(&http, &addr, "wrong").await.status(), 401);
    assert_eq!(
        locate_once(&http, &addr, &unknown_key()).await.status(),
        401
    );
}

#[tokio::test]
async fn locate_returns_offline_then_phone_fix() {
    let addr = start_relay().await;
    let http = reqwest::Client::new();
    let key = agent_key_a();
    assert_eq!(locate_once(&http, &addr, &key).await.status(), 503);

    connect_phone(&addr, "phone-a", 32.08).await;
    let res = locate_settled(&http, &addr, &key).await;
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["latitude"], 32.08);
    assert_eq!(body["longitude"], 34.78);
}
