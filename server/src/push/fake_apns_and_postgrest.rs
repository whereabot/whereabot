//! Test double: one server plays PostgREST (two tokens, `bb22` is dead; `trusted-agent` is
//! always allowed; sharing RPCs in `fake_share_rpcs`), the grants list, and APNs.

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use super::{Push, PushConfig};
use axum::{
    Json, Router,
    extract::{Path, RawQuery, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use p256::{
    SecretKey,
    elliptic_curve::Generate,
    pkcs8::{EncodePrivateKey, LineEnding},
};
use serde_json::{Value, json};

#[derive(Default)]
pub struct Fake {
    /// (device token, apns headers, body) for each push received.
    pub pushes: Mutex<Vec<(String, HeaderMap, Value)>>,
    pub deleted: Mutex<Vec<String>>,
}

type Shared = State<Arc<Fake>>;

async fn list() -> Json<Value> {
    Json(json!([{ "token": "aa11" }, { "token": "bb22" }]))
}

async fn grants() -> Json<Value> {
    Json(json!([
        { "client": { "id": "claude-code", "name": "Claude Code" } },
        { "client": { "id": "trusted-agent", "name": "Trusted Bot" } },
    ]))
}

async fn always(RawQuery(q): RawQuery) -> Json<Value> {
    let hit = q.unwrap_or_default().contains("client_id=eq.trusted-agent");
    Json(if hit {
        json!([{ "client_id": "trusted-agent" }])
    } else {
        json!([])
    })
}

async fn drop_token(State(f): Shared, RawQuery(q): RawQuery) {
    f.deleted.lock().unwrap().push(q.unwrap_or_default());
}

async fn apns(
    State(f): Shared,
    Path(token): Path<String>,
    h: HeaderMap,
    Json(b): Json<Value>,
) -> impl IntoResponse {
    let gone = token == "bb22";
    f.pushes.lock().unwrap().push((token, h, b));
    if gone {
        (StatusCode::GONE, Json(json!({ "reason": "Unregistered" }))).into_response()
    } else {
        StatusCode::OK.into_response()
    }
}

pub async fn start() -> (Push, Arc<Fake>) {
    let fake = Arc::new(Fake::default());
    let app = Router::new()
        .route("/rest/v1/push_tokens", get(list).delete(drop_token))
        .route("/rest/v1/always_allowed_agents", get(always))
        .route("/auth/v1/user/oauth/grants", get(grants))
        .route("/3/device/{token}", post(apns))
        .merge(super::fake_share_rpcs::routes())
        .with_state(fake.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let key_pem = SecretKey::generate()
        .to_pkcs8_pem(LineEnding::LF)
        .unwrap()
        .to_string();
    let config = PushConfig {
        key_pem,
        key_id: "KEYID".into(),
        team_id: "TEAMID".into(),
        topic: "io.whereabot.app".into(),
        apns_url: url.clone(),
        supabase_url: url,
        anon_key: "anon".into(),
    };
    let mut push = Push::new(&config).unwrap();
    push.silent_window = Duration::from_millis(300);
    (push, fake)
}
