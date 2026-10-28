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

use crate::{
    AppState, agent_bearer_auth, devices::device_revocation_check,
    oauth_protected_resource_metadata, phone_message_handler, relay_wire_protocol::ToDevice,
};

pub async fn device(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    let Some(token) = agent_bearer_auth::verify(&state, &headers, false).await else {
        return oauth_protected_resource_metadata::unauthorized(&state.public_url, &headers);
    };
    let bearer = agent_bearer_auth::bearer(&headers)
        .unwrap_or_default()
        .to_string();
    ws.on_upgrade(move |socket| serve_device(state, token.user_id, bearer, socket))
}

async fn serve_device(state: Arc<AppState>, user_id: Uuid, bearer: String, mut socket: WebSocket) {
    let (tx, mut rx) = mpsc::channel(8);
    // Until `hello` names the device_id, the socket lives under a random key.
    let mut key = Uuid::new_v4();
    state.devices.insert(user_id, key, tx.clone()).await;
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
                Some(Ok(Message::Text(text))) => {
                    let Some(id) = phone_message_handler::handle(&state, user_id, key, &text).await else {
                        continue;
                    };
                    if device_revocation_check::is_revoked(&state, &bearer, id).await {
                        let frame = CloseFrame { code: 4000, reason: "revoked".into() };
                        let _ = socket.send(Message::Close(Some(frame))).await;
                        break;
                    }
                    state.devices.rekey(user_id, key, id, &tx).await;
                    key = id;
                }
                Some(Ok(_)) => continue,
                _ => break,
            },
            else => break,
        }
    }
    tracing::info!(%user_id, "device disconnected");
    state.devices.remove(user_id, key, &tx).await;
}
