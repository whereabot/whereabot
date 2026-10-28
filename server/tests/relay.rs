use futures_util::{SinkExt, StreamExt};
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
            let ping: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(ping["type"], "locate");
            let reply = serde_json::json!({
                "type": "location", "id": ping["id"],
                "latitude": 32.08, "longitude": 34.78, "accuracy_m": 5.0, "captured_at_ms": 1
            });
            ws.send(Message::Text(reply.to_string().into()))
                .await
                .unwrap();
        }
    });
}

#[tokio::test]
async fn agent_ping_reaches_device_and_returns_location() {
    let addr = start().await;
    let http = reqwest::Client::new();
    let url = format!("http://{addr}/v1/locate?timeout_secs=5");

    let offline = http.post(&url).bearer_auth("agent").send().await.unwrap();
    assert_eq!(offline.status(), 503);

    let bad = http.post(&url).bearer_auth("wrong").send().await.unwrap();
    assert_eq!(bad.status(), 401);

    connect_device(&addr).await;
    // The server registers the socket just after the handshake, so retry briefly.
    let mut res = http.post(&url).bearer_auth("agent").send().await.unwrap();
    for _ in 0..50 {
        if res.status() != 503 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        res = http.post(&url).bearer_auth("agent").send().await.unwrap();
    }
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["latitude"], 32.08);
    assert_eq!(body["longitude"], 34.78);
}

#[tokio::test]
async fn device_needs_token() {
    let addr = start().await;
    let err = tokio_tungstenite::connect_async(format!("ws://{addr}/v1/device")).await;
    assert!(err.is_err());
}
