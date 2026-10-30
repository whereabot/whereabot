mod common;

use common::{fake_supabase::USER_A, mcp_client::rpc, start_relay};
use serde_json::{Value, json};

fn tool<'a>(tools: &'a [Value], name: &str) -> &'a Value {
    tools.iter().find(|tool| tool["name"] == name).unwrap()
}

#[tokio::test]
async fn tools_list_carries_titles_and_annotations() {
    let relay = start_relay().await;
    let list = rpc(
        &reqwest::Client::new(),
        &relay.addr,
        &relay.mcp(USER_A),
        json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }),
    )
    .await;
    let tools = list["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 2);

    let location = tool(tools, "get_phone_location");
    assert_eq!(location["title"], "Get phone location");
    assert_eq!(
        location["annotations"],
        json!({
            "title": "Get phone location",
            "readOnlyHint": true,
            "destructiveHint": false,
            "idempotentHint": false,
            "openWorldHint": true
        })
    );

    let people = tool(tools, "list_people_sharing_with_me");
    assert_eq!(people["title"], "List people sharing with me");
    assert_eq!(
        people["annotations"],
        json!({
            "title": "List people sharing with me",
            "readOnlyHint": true,
            "destructiveHint": false,
            "idempotentHint": true,
            "openWorldHint": false
        })
    );
}
