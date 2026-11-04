mod common;

use common::{
    fake_supabase::USER_A, mcp_client::rpc, scripted_fake_phone::connect_scripted, start_relay,
};
use serde_json::{Value, json};

fn call_with_reason(reason: Value) -> Value {
    json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "get_phone_location", "arguments": { "timeout_secs": 5, "reason": reason } }
    })
}

/// Pings the phone until the socket registers, and returns the `locate` ping it got.
async fn ping_for(reason: Value) -> Value {
    let relay = start_relay().await;
    let http = reqwest::Client::new();
    let mut pings = connect_scripted(&relay.addr, &relay.phone(USER_A), None, |ping| {
        Some(json!({
            "type": "location", "id": ping["id"],
            "latitude": 32.08, "longitude": 34.78, "accuracy_m": 5.0, "captured_at_ms": 1
        }))
    })
    .await;
    let key = relay.mcp(USER_A);
    for _ in 0..50 {
        let res = rpc(&http, &relay.addr, &key, call_with_reason(reason.clone())).await;
        if res["result"]["isError"] != true {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    pings.recv().await.unwrap()
}

#[tokio::test]
async fn the_phone_gets_the_agents_reason_cleaned() {
    let ping = ping_for(json!("  To find\ncoffee near you ")).await;
    assert_eq!(ping["type"], "locate");
    assert_eq!(ping["reason"], "To find coffee near you");
}

#[tokio::test]
async fn a_blank_reason_reaches_the_phone_as_null() {
    let ping = ping_for(json!("   ")).await;
    assert!(ping["reason"].is_null());
}
