//! The cap on open locate requests per owner, and that every exit frees a slot.

mod common;

use reqwest::StatusCode;
use serde_json::Value;
use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};

use common::{
    Relay, fake_supabase::USER_A, locate_limit_fixtures::limits, locate_once,
    scripted_fake_phone::connect_scripted, spawn_with,
};

/// Starts a locate for `token` that waits up to 120 s, and returns once the phone got its ping.
async fn open_request(
    relay: &Relay,
    token: &str,
    pings: &mut UnboundedReceiver<Value>,
) -> JoinHandle<StatusCode> {
    let request = reqwest::Client::new()
        .post(format!("http://{}/v1/locate?timeout_secs=120", relay.addr))
        .bearer_auth(token);
    let task = tokio::spawn(async move { request.send().await.unwrap().status() });
    // The ping proves the request holds its slot: no fixed sleep needed.
    assert_eq!(pings.recv().await.unwrap()["type"], "locate");
    task
}

#[tokio::test]
async fn pending_requests_per_owner_are_capped() {
    let relay = spawn_with(false, limits(1, 10, 1)).await;
    let http = reqwest::Client::new();
    let token = relay.mcp(USER_A);
    let mut pings = connect_scripted(&relay.addr, &relay.phone(USER_A), None, |_| None).await;
    let _first = open_request(&relay, &token, &mut pings).await;
    let res = locate_once(&http, &relay.addr, &token).await;
    assert_eq!(res.status(), 429);
    assert!(res.headers().contains_key("retry-after"));
}

#[tokio::test]
async fn cancelled_request_frees_its_slot() {
    let relay = spawn_with(false, limits(1, 10, 1)).await;
    let http = reqwest::Client::new();
    let token = relay.mcp(USER_A);
    let mut pings = connect_scripted(&relay.addr, &relay.phone(USER_A), None, |_| None).await;
    // The caller hangs up long before the 120 s timeout.
    open_request(&relay, &token, &mut pings).await.abort();
    // Poll until the relay sees the closed connection; a leaked slot keeps answering 429.
    let mut status = StatusCode::TOO_MANY_REQUESTS;
    for _ in 0..100 {
        status = locate_once(&http, &relay.addr, &token).await.status();
        if status != StatusCode::TOO_MANY_REQUESTS {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    // The silent phone never answers, so the new request times out instead of 429.
    assert_eq!(status, StatusCode::GATEWAY_TIMEOUT);
}
