use std::{sync::Arc, time::Duration};

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{AppState, auth, protocol::FromDevice};

pub async fn device(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    let Some(user_id) = auth::device_user(&state, &headers).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    ws.on_upgrade(move |socket| serve_device(state, user_id, socket))
}

async fn serve_device(state: Arc<AppState>, user_id: Uuid, mut socket: WebSocket) {
    let (tx, mut rx) = mpsc::channel(8);
    // A new connection replaces the user's old one; the old loop ends when its sender drops.
    state.devices.lock().await.insert(user_id, tx.clone());
    tracing::info!(%user_id, "device connected");
    // Keeps mobile networks and Cloudflare from dropping an idle socket.
    let mut keepalive = tokio::time::interval(Duration::from_secs(25));
    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                let text = serde_json::to_string(&msg).expect("ToDevice serializes");
                if socket.send(Message::Text(text.into())).await.is_err() {
                    break;
                }
            }
            _ = keepalive.tick() => {
                if socket.send(Message::Ping(Default::default())).await.is_err() {
                    break;
                }
            }
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Text(text))) => match serde_json::from_str(&text) {
                    Ok(FromDevice::Location(location)) => {
                        if let Some(waiter) = state.pending.lock().await.remove(&location.id) {
                            let _ = waiter.send(location);
                        }
                    }
                    Err(_) => continue,
                },
                Some(Ok(_)) => continue,
                _ => break,
            },
            else => break,
        }
    }
    tracing::info!(%user_id, "device disconnected");
    let mut devices = state.devices.lock().await;
    if devices
        .get(&user_id)
        .is_some_and(|current| current.same_channel(&tx))
    {
        devices.remove(&user_id);
    }
}
