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

pub const DEVICE_1: &str = "11111111-1111-4111-8111-111111111111";
pub const DEVICE_2: &str = "22222222-2222-4222-8222-222222222222";

type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// Sends the `hello` that names this install.
pub async fn hello(ws: &mut Ws, device_id: &str) {
    let msg = json!({
        "type": "hello", "app_version": "0.2.0", "device_id": device_id,
        "device_name": "Test phone", "platform": "ios"
    });
    ws.send(Message::Text(msg.to_string().into()))
        .await
        .unwrap();
}

/// The close code and reason of the next Close frame, within 5 s.
pub async fn next_close(ws: &mut Ws) -> (u16, String) {
    let wait = async {
        while let Some(Ok(msg)) = ws.next().await {
            if let Message::Close(Some(frame)) = msg {
                return (u16::from(frame.code), frame.reason.to_string());
            }
        }
        panic!("socket ended without a Close frame");
    };
    tokio::time::timeout(std::time::Duration::from_secs(5), wait)
        .await
        .expect("no Close frame")
}
