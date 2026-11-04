//! Shared by the push test doubles: a loopback server, the user's OAuth grants (agent names),
//! and the always-allowed rows (`trusted-agent` only).

use axum::{Json, Router, extract::RawQuery};
use serde_json::{Value, json};

pub async fn grants() -> Json<Value> {
    Json(json!([
        { "client": { "id": "claude-code", "name": "Claude Code" } },
        { "client": { "id": "trusted-agent", "name": "Trusted Bot" } },
    ]))
}

pub async fn always(RawQuery(q): RawQuery) -> Json<Value> {
    let hit = q.unwrap_or_default().contains("client_id=eq.trusted-agent");
    Json(if hit {
        json!([{ "client_id": "trusted-agent" }])
    } else {
        json!([])
    })
}

/// Serves `app` on a free loopback port; returns its base URL.
pub async fn serve(app: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    url
}
