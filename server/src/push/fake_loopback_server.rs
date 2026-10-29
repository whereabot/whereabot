//! Shared by the push test doubles: a loopback server and the user's OAuth grants (agent names).

use axum::{Json, Router};
use serde_json::{Value, json};

pub async fn grants() -> Json<Value> {
    Json(json!([
        { "client": { "id": "claude-code", "name": "Claude Code" } },
        { "client": { "id": "trusted-agent", "name": "Trusted Bot" } },
    ]))
}

/// Serves `app` on a free loopback port; returns its base URL.
pub async fn serve(app: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    url
}
