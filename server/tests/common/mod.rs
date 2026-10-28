#![allow(dead_code)]
//! Shared fixtures: relay start-up and the agent-side HTTP helpers.

pub mod mcp;
pub mod phone;
pub mod supabase;

use std::time::Duration;

use tokio::net::TcpListener;
use whereabouts_server::{Config, app};

/// Starts a fake Supabase and a relay wired to it. Returns the relay address.
pub async fn start_relay() -> String {
    let supabase_url = supabase::start().await;
    let config = Config {
        supabase_url,
        anon_key: supabase::ANON_KEY.into(),
        service_role_key: supabase::SERVICE_KEY.into(),
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app(config)).await.unwrap() });
    addr.to_string()
}

/// One `/v1/locate` call with the given agent bearer token.
pub async fn locate_once(http: &reqwest::Client, addr: &str, key: &str) -> reqwest::Response {
    http.post(format!("http://{addr}/v1/locate?timeout_secs=5"))
        .bearer_auth(key)
        .send()
        .await
        .unwrap()
}

/// Retries `locate_once` while the phone is offline. Sockets register just after the handshake.
pub async fn locate_settled(http: &reqwest::Client, addr: &str, key: &str) -> reqwest::Response {
    let mut res = locate_once(http, addr, key).await;
    for _ in 0..50 {
        if res.status() != 503 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
        res = locate_once(http, addr, key).await;
    }
    res
}
