use std::{collections::HashMap, sync::Arc, time::Duration};

use axum::{
    Json, Router,
    extract::{
        Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    response::{IntoResponse, Response},
    routing::{any, get, post},
};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use tokio::sync::{Mutex, mpsc, oneshot};
use uuid::Uuid;

/// Fix sent by the phone in answer to a `locate` ping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_m: Option<f64>,
    pub captured_at_ms: u64,
}

/// Server -> device.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ToDevice {
    Locate { id: Uuid },
}

/// Device -> server.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum FromDevice {
    Location(Location),
}

pub struct Config {
    pub agent_token: String,
    pub device_token: String,
}

struct AppState {
    config: Config,
    // shortcut: one paired device per server, add a device registry for multi-device.
    device: Mutex<Option<mpsc::Sender<ToDevice>>>,
    pending: Mutex<HashMap<Uuid, oneshot::Sender<Location>>>,
}

pub fn app(config: Config) -> Router {
    let state = Arc::new(AppState {
        config,
        device: Mutex::default(),
        pending: Mutex::default(),
    });
    Router::new()
        .route("/v1/locate", post(locate))
        .route("/v1/device", any(device))
        .route("/healthz", get(healthz))
        .with_state(state)
}

async fn healthz() -> &'static str {
    "ok"
}

fn authorized(headers: &HeaderMap, token: &str) -> bool {
    headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .is_some_and(|given| bool::from(given.as_bytes().ct_eq(token.as_bytes())))
}

#[derive(Deserialize)]
struct LocateParams {
    timeout_secs: Option<u64>,
}

async fn locate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<LocateParams>,
) -> Response {
    if !authorized(&headers, &state.config.agent_token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let Some(device) = state.device.lock().await.clone() else {
        return (StatusCode::SERVICE_UNAVAILABLE, "device offline").into_response();
    };
    let id = Uuid::new_v4();
    let (tx, rx) = oneshot::channel();
    state.pending.lock().await.insert(id, tx);
    if device.send(ToDevice::Locate { id }).await.is_err() {
        state.pending.lock().await.remove(&id);
        return (StatusCode::SERVICE_UNAVAILABLE, "device offline").into_response();
    }
    let timeout = Duration::from_secs(params.timeout_secs.unwrap_or(30).min(120));
    match tokio::time::timeout(timeout, rx).await {
        Ok(Ok(location)) => Json(location).into_response(),
        _ => {
            state.pending.lock().await.remove(&id);
            (StatusCode::GATEWAY_TIMEOUT, "device did not answer").into_response()
        }
    }
}

async fn device(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    if !authorized(&headers, &state.config.device_token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    ws.on_upgrade(move |socket| serve_device(state, socket))
}

async fn serve_device(state: Arc<AppState>, mut socket: WebSocket) {
    let (tx, mut rx) = mpsc::channel(8);
    // A new connection replaces the old one; the old loop ends when its sender drops.
    *state.device.lock().await = Some(tx.clone());
    tracing::info!("device connected");
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
    tracing::info!("device disconnected");
    let mut device = state.device.lock().await;
    if device
        .as_ref()
        .is_some_and(|current| current.same_channel(&tx))
    {
        *device = None;
    }
}
