mod common;

use common::{CHALLENGE, agent_rpc::agent_key_a, start_relay, supabase::USER_A};
use serde_json::json;

const ACCEPT: &str = "application/json, text/event-stream";

async fn list_tools(addr: &str, token: Option<&str>) -> reqwest::Response {
    let mut req = reqwest::Client::new()
        .post(format!("http://{addr}/mcp"))
        .header("Accept", ACCEPT)
        .json(&json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }));
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    req.send().await.unwrap()
}

fn challenge(res: &reqwest::Response) -> &str {
    res.headers()["www-authenticate"].to_str().unwrap()
}

#[tokio::test]
async fn mcp_without_token_gets_challenge() {
    let relay = start_relay().await;
    let res = list_tools(&relay.addr, None).await;
    assert_eq!(res.status(), 401);
    assert_eq!(challenge(&res), CHALLENGE);
}

#[tokio::test]
async fn mcp_accepts_oauth_client_token() {
    let relay = start_relay().await;
    let res = list_tools(&relay.addr, Some(&relay.mcp(USER_A))).await;
    assert_eq!(res.status(), 200);
}

#[tokio::test]
async fn mcp_rejects_phone_token_without_client_id() {
    let relay = start_relay().await;
    let res = list_tools(&relay.addr, Some(&relay.phone(USER_A))).await;
    assert_eq!(res.status(), 401);
    assert_eq!(challenge(&res), CHALLENGE);
}

#[tokio::test]
async fn mcp_rejects_agent_key_when_fallback_off() {
    let relay = start_relay().await;
    let res = list_tools(&relay.addr, Some(&agent_key_a())).await;
    assert_eq!(res.status(), 401);
    assert_eq!(challenge(&res), CHALLENGE);
}
