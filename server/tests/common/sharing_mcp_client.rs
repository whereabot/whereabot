#![allow(dead_code)]
//! Helpers for sharing tests: share rows, tool calls as USER_A, and owner phones.

use std::time::Duration;

use serde_json::{Value, json};

use super::{
    Relay, fake_location_shares::ShareRow, fake_supabase::USER_A, mcp_client::rpc,
    scripted_fake_phone::connect_scripted,
};

/// A share from `owner` to USER_A; `revoked` marks it stopped.
pub fn share(
    owner: &'static str,
    name: &'static str,
    email: &'static str,
    revoked: bool,
) -> ShareRow {
    let grantee = USER_A;
    ShareRow {
        owner,
        grantee,
        name,
        email,
        revoked,
    }
}

/// Calls a tool as USER_A (with an email claim) and returns (is_error, text).
pub async fn call(relay: &Relay, tool: &str, arguments: Value) -> (bool, String) {
    let body = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call",
                       "params": { "name": tool, "arguments": arguments } });
    let key = relay.mcp_email(USER_A, "ofek@example.com");
    let res = rpc(&reqwest::Client::new(), &relay.addr, &key, body).await;
    let text = res["result"]["content"][0]["text"].as_str().unwrap();
    (res["result"]["isError"] == true, text.to_owned())
}

/// Connects a phone of `user` that answers with `reply` and returns the pings it gets.
pub async fn phone(
    relay: &Relay,
    user: &str,
    reply: &'static str,
) -> tokio::sync::mpsc::UnboundedReceiver<Value> {
    let answer = move |ping: &Value| {
        Some(
            json!({ "type": reply, "id": ping["id"], "latitude": 1.5, "longitude": 2.5,
                     "accuracy_m": 5.0, "captured_at_ms": 1 }),
        )
    };
    let pings = connect_scripted(&relay.addr, &relay.phone(user), None, answer).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    pings
}
