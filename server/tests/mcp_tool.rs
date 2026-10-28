mod common;

use common::{
    mcp::{OFFLINE, call_location, call_settled, rpc},
    phone::connect_phone,
    start_relay,
    supabase::USER_A,
};
use serde_json::{Value, json};

#[tokio::test]
async fn mcp_lists_and_calls_location_tool() {
    let relay = start_relay().await;
    let addr = relay.addr.clone();
    let http = reqwest::Client::new();
    let key = relay.mcp(USER_A);

    let init = rpc(
        &http,
        &addr,
        &key,
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18", "capabilities": {},
                "clientInfo": { "name": "test", "version": "1.0" }
            }
        }),
    )
    .await;
    assert!(init["result"]["capabilities"]["tools"].is_object());

    let list = rpc(
        &http,
        &addr,
        &key,
        json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }),
    )
    .await;
    let tools = list["result"]["tools"].as_array().unwrap();
    assert!(
        tools
            .iter()
            .any(|tool| tool["name"] == "get_phone_location")
    );

    // No phone yet: the tool reports an error result, not a JSON-RPC error.
    let offline = rpc(&http, &addr, &key, call_location(3)).await;
    assert_eq!(offline["result"]["isError"], true);
    assert_eq!(offline["result"]["content"][0]["text"], OFFLINE);

    connect_phone(&addr, &relay.phone(USER_A), 32.08).await;
    let res = call_settled(&http, &addr, &key, 4).await;
    assert!(res["result"]["isError"].is_null() || res["result"]["isError"] == false);
    let text = res["result"]["content"][0]["text"].as_str().unwrap();
    let location: Value = serde_json::from_str(text).unwrap();
    assert_eq!(location["latitude"], 32.08);
    assert_eq!(location["longitude"], 34.78);
}
