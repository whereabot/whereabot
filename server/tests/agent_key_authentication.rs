mod common;

use common::{
    agent_key_rpc_fake::agent_key_a, fake_phone::connect_phone, fake_supabase::USER_A, locate_once,
    locate_settled, mcp_client::call_settled, spawn, start_relay,
};

#[tokio::test]
async fn agent_key_rejected_when_fallback_off() {
    let relay = start_relay().await;
    let http = reqwest::Client::new();
    connect_phone(&relay.addr, &relay.phone(USER_A), 32.08).await;
    assert_eq!(
        locate_once(&http, &relay.addr, &agent_key_a())
            .await
            .status(),
        401
    );
    let res = http
        .post(format!("http://{}/mcp", relay.addr))
        .bearer_auth(agent_key_a())
        .header("Accept", "application/json, text/event-stream")
        .json(&serde_json::json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn agent_key_reaches_locate_and_mcp_when_fallback_on() {
    let relay = spawn(true).await;
    let http = reqwest::Client::new();
    connect_phone(&relay.addr, &relay.phone(USER_A), 32.08).await;

    let locate = locate_settled(&http, &relay.addr, &agent_key_a()).await;
    assert_eq!(locate.status(), 200);

    let mcp = call_settled(&http, &relay.addr, &agent_key_a(), 3).await;
    assert!(mcp["result"]["isError"] != true);
}
