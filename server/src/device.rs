use std::{sync::Arc, time::Duration};

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{CloseFrame, Message, WebSocket},
    },
    http::HeaderMap,
    response::Response,
};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{AppState, auth, device_in, oauth, protocol::ToDevice};

pub async fn device(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    let Some(token) = auth::verify(&state, &headers, false).await else {
        return oauth::unauthorized(&state.public_url);
    };
    ws.on_upgrade(move |socket| serve_device(state, token.user_id, socket))
}

async fn serve_device(state: Arc<AppState>, user_id: Uuid, mut socket: WebSocket) {
    let (tx, mut rx) = mpsc::channel(8);
    // A new connection replaces the user's old one. This loop keeps its own `tx` to
    // tell whether it is still current, so the old loop ends on an explicit Close.
    let old = state.devices.lock().await.insert(user_id, tx.clone());
    if let Some(old) = old {
        let _ = old.try_send(ToDevice::Close {
            code: 4000,
            reason: "replaced",
        });
    }
    tracing::info!(%user_id, "device connected");
    // Keeps mobile networks and Cloudflare from dropping an idle socket.
    let mut keepalive = tokio::time::interval(Duration::from_secs(25));
    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                if let ToDevice::Close { code, reason } = msg {
                    let frame = CloseFrame { code, reason: reason.into() };
                    let _ = socket.send(Message::Close(Some(frame))).await;
                    break;
                }
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
                Some(Ok(Message::Text(text))) => device_in::handle(&state, user_id, &text).await,
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
