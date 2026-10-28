#![allow(dead_code)]
//! Agent-side HTTP helpers for `/v1/locate`.

use std::time::Duration;

/// One `/v1/locate` call with the given bearer token.
pub async fn locate_once(http: &reqwest::Client, addr: &str, token: &str) -> reqwest::Response {
    http.post(format!("http://{addr}/v1/locate?timeout_secs=5"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

/// Retries `locate_once` while the phone is offline. Sockets register just after the handshake.
pub async fn locate_settled(http: &reqwest::Client, addr: &str, token: &str) -> reqwest::Response {
    let mut res = locate_once(http, addr, token).await;
    for _ in 0..50 {
        if res.status() != 503 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
        res = locate_once(http, addr, token).await;
    }
    res
}
