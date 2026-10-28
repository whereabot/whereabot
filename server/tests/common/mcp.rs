#![allow(dead_code)]
//! MCP client helpers for /mcp.

use serde_json::{Value, json};

pub const OFFLINE: &str = "The user's phone is not connected. Ask the user to open the Whereabot app and keep it open, then retry.";

/// Sends one JSON-RPC message to /mcp with the given bearer token. Returns the decoded reply.
pub async fn rpc(http: &reqwest::Client, addr: &str, key: &str, body: Value) -> Value {
    let res = http
        .post(format!("http://{addr}/mcp"))
        .bearer_auth(key)
        .header("Accept", "application/json, text/event-stream")
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let text = res.text().await.unwrap();
    match text.lines().find_map(|line| line.strip_prefix("data:")) {
        Some(data) => serde_json::from_str(data.trim()).unwrap(),
        None => serde_json::from_str(&text).unwrap(),
    }
}

pub fn call_location(id: u64) -> Value {
    json!({
        "jsonrpc": "2.0", "id": id, "method": "tools/call",
        "params": { "name": "get_phone_location", "arguments": { "timeout_secs": 5 } }
    })
}

/// Calls the tool while it reports an error, since sockets register just after the handshake.
pub async fn call_settled(http: &reqwest::Client, addr: &str, key: &str, id: u64) -> Value {
    let mut res = rpc(http, addr, key, call_location(id)).await;
    for _ in 0..50 {
        if res["result"]["isError"] != true {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        res = rpc(http, addr, key, call_location(id)).await;
    }
    res
}
