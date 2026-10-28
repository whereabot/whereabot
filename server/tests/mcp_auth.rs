mod common;

use common::{
    start_relay,
    supabase::{agent_key_a, unknown_key},
};
use serde_json::json;

const ACCEPT: &str = "application/json, text/event-stream";

#[tokio::test]
async fn mcp_rejects_missing_or_unknown_key() {
    let addr = start_relay().await;
    let http = reqwest::Client::new();
    let url = format!("http://{addr}/mcp");
    let body = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" });

    let none = http
        .post(&url)
        .header("Accept", ACCEPT)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(none.status(), 401);

    let unknown = http
        .post(&url)
        .bearer_auth(unknown_key())
        .header("Accept", ACCEPT)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(unknown.status(), 401);

    // A phone token is not an agent key.
    let phone = http
        .post(&url)
        .bearer_auth("phone-a")
        .header("Accept", ACCEPT)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(phone.status(), 401);

    let ok = http
        .post(&url)
        .bearer_auth(agent_key_a())
        .header("Accept", ACCEPT)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(ok.status(), 200);
}
