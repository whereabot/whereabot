#![allow(dead_code)]
//! Fake phone: a WebSocket client that answers every locate ping.

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::{Message, client::IntoClientRequest};

/// Connects a phone with the given access token. It answers every ping with `latitude`.
pub async fn connect_phone(addr: &str, token: &str, latitude: f64) {
    let mut req = format!("ws://{addr}/v1/device")
        .into_client_request()
        .unwrap();
    req.headers_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    let (mut ws, _) = tokio_tungstenite::connect_async(req).await.unwrap();
    tokio::spawn(async move {
        while let Some(Ok(msg)) = ws.next().await {
            // Keepalive pings are not text frames; skip them.
            let Message::Text(text) = msg else { continue };
            let ping: Value = serde_json::from_str(&text).unwrap();
            assert_eq!(ping["type"], "locate");
            let reply = json!({
                "type": "location", "id": ping["id"],
                "latitude": latitude, "longitude": 34.78, "accuracy_m": 5.0, "captured_at_ms": 1
            });
            ws.send(Message::Text(reply.to_string().into()))
                .await
                .unwrap();
        }
    });
}

/// Connects a phone that never answers. Drop the stream to disconnect it.
pub async fn connect_silent_phone(
    addr: &str,
    token: &str,
) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
    let mut req = format!("ws://{addr}/v1/device")
        .into_client_request()
        .unwrap();
    req.headers_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    tokio_tungstenite::connect_async(req).await.unwrap().0
}

/// Connects a phone whose user declines every ping. It asserts the ping names `client_id`.
pub async fn connect_denying_phone(addr: &str, token: &str, client_id: &str) {
    let mut req = format!("ws://{addr}/v1/device")
        .into_client_request()
        .unwrap();
    req.headers_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    let (mut ws, _) = tokio_tungstenite::connect_async(req).await.unwrap();
    let client_id = client_id.to_string();
    tokio::spawn(async move {
        while let Some(Ok(msg)) = ws.next().await {
            let Message::Text(text) = msg else { continue };
            let ping: Value = serde_json::from_str(&text).unwrap();
            assert_eq!(ping["client_id"], client_id.as_str());
            let reply = json!({ "type": "denied", "id": ping["id"] });
            ws.send(Message::Text(reply.to_string().into()))
                .await
                .unwrap();
        }
    });
}
