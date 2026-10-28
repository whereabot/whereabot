mod common;

use common::{
    locate_settled, mcp::call_settled, phone::connect_denying_phone, start_relay, supabase::USER_A,
};

#[tokio::test]
async fn locate_returns_403_when_user_declines() {
    let relay = start_relay().await;
    let http = reqwest::Client::new();
    connect_denying_phone(&relay.addr, &relay.phone(USER_A), "claude-code").await;
    let res = locate_settled(&http, &relay.addr, &relay.mcp(USER_A)).await;
    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn mcp_tool_errors_when_user_declines() {
    let relay = start_relay().await;
    let http = reqwest::Client::new();
    connect_denying_phone(&relay.addr, &relay.phone(USER_A), "claude-code").await;
    let res = call_settled(&http, &relay.addr, &relay.mcp(USER_A), 7).await;
    assert_eq!(res["result"]["isError"], true);
    assert_eq!(
        res["result"]["content"][0]["text"],
        "The user declined to share their location."
    );
}
