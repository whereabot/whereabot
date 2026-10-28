mod common;

use common::{
    locate_once, locate_settled,
    mcp::{OFFLINE, call_location, call_settled, rpc},
    phone::connect_phone,
    start_relay,
    supabase::{agent_key_a, agent_key_b},
};
use serde_json::Value;

#[tokio::test]
async fn agent_b_never_reaches_phone_a() {
    let addr = start_relay().await;
    let http = reqwest::Client::new();
    connect_phone(&addr, "phone-a", 32.08).await;
    let a = locate_settled(&http, &addr, &agent_key_a()).await;
    assert_eq!(a.status(), 200);

    // Only phone A is connected. Agent B must get no fix from it.
    assert_eq!(
        locate_once(&http, &addr, &agent_key_b()).await.status(),
        503
    );

    connect_phone(&addr, "phone-b", 51.5).await;
    let b: Value = locate_settled(&http, &addr, &agent_key_b())
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(b["latitude"], 51.5);
    let a: Value = locate_once(&http, &addr, &agent_key_a())
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(a["latitude"], 32.08);
}

#[tokio::test]
async fn mcp_agent_b_never_reaches_phone_a() {
    let addr = start_relay().await;
    let http = reqwest::Client::new();
    connect_phone(&addr, "phone-a", 32.08).await;
    let a = call_settled(&http, &addr, &agent_key_a(), 3).await;
    assert!(a["result"]["isError"] != true);

    let b = rpc(&http, &addr, &agent_key_b(), call_location(4)).await;
    assert_eq!(b["result"]["isError"], true);
    assert_eq!(b["result"]["content"][0]["text"], OFFLINE);
}
