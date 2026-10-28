use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::{Message, client::IntoClientRequest};
use whereabouts_server::{Config, app};

async fn start() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let config = Config {
        agent_token: "agent".into(),
        device_token: "device".into(),
    };
    tokio::spawn(async move { axum::serve(listener, app(config)).await.unwrap() });
    addr.to_string()
}

async fn connect_device(addr: &str) {
    let mut req = format!("ws://{addr}/v1/device")
        .into_client_request()
        .unwrap();
    req.headers_mut()
        .insert("authorization", "Bearer device".parse().unwrap());
    let (mut ws, _) = tokio_tungstenite::connect_async(req).await.unwrap();
    // Fake phone: answer every ping with a fixed fix.
    tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = ws.next().await {
            let ping: Value = serde_json::from_str(&text).unwrap();
            assert_eq!(ping["type"], "locate");
            let reply = json!({
                "type": "location", "id": ping["id"],
                "latitude": 32.08, "longitude": 34.78, "accuracy_m": 5.0, "captured_at_ms": 1
            });
            ws.send(Message::Text(reply.to_string().into()))
                .await
                .unwrap();
        }
    });
}

/// Sends one JSON-RPC message to /mcp and returns the decoded response (JSON or SSE).
async fn rpc(http: &reqwest::Client, addr: &str, body: Value) -> Value {
    let res = http
        .post(format!("http://{addr}/mcp"))
        .bearer_auth("agent")
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

fn call_location(id: u64) -> Value {
    json!({
        "jsonrpc": "2.0", "id": id, "method": "tools/call",
        "params": { "name": "get_phone_location", "arguments": { "timeout_secs": 5 } }
    })
}

#[tokio::test]
async fn mcp_requires_agent_token() {
    let addr = start().await;
    let body = json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/list"
    });
    let res = reqwest::Client::new()
        .post(format!("http://{addr}/mcp"))
        .header("Accept", "application/json, text/event-stream")
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn mcp_lists_and_calls_location_tool() {
    let addr = start().await;
    let http = reqwest::Client::new();

    let init = rpc(
        &http,
        &addr,
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
    let offline = rpc(&http, &addr, call_location(3)).await;
    assert_eq!(offline["result"]["isError"], true);
    assert_eq!(
        offline["result"]["content"][0]["text"],
        "The phone is not connected to the relay. Ask the user to open the Whereabouts app."
    );

    connect_device(&addr).await;
    // The server registers the socket just after the handshake, so retry briefly.
    let mut res = rpc(&http, &addr, call_location(4)).await;
    for _ in 0..50 {
        if res["result"]["isError"] != true {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        res = rpc(&http, &addr, call_location(4)).await;
    }
    assert!(res["result"]["isError"].is_null() || res["result"]["isError"] == false);
    let text = res["result"]["content"][0]["text"].as_str().unwrap();
    let location: Value = serde_json::from_str(text).unwrap();
    assert_eq!(location["latitude"], 32.08);
    assert_eq!(location["longitude"], 34.78);
}
