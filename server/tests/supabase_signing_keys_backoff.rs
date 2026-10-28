mod common;

use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

use common::{
    Relay,
    fake_supabase::USER_A,
    spawn,
    test_token_signer::{claims, sign_with_kid},
};
use serde_json::json;

const ACCEPT: &str = "application/json, text/event-stream";

/// HTTP status of `/mcp` with the given token.
async fn status(relay: &Relay, token: &str) -> u16 {
    let body = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" });
    reqwest::Client::new()
        .post(format!("http://{}/mcp", relay.addr))
        .bearer_auth(token)
        .header("Accept", ACCEPT)
        .json(&body)
        .send()
        .await
        .unwrap()
        .status()
        .as_u16()
}

#[tokio::test]
async fn failed_jwks_fetch_backs_off_for_30s() {
    let relay = spawn(false).await;
    relay.jwks_fail.store(true, Ordering::SeqCst);
    let token = relay.mcp(USER_A);
    assert_eq!(status(&relay, &token).await, 401);

    // Inside the 30 s gap: no second fetch, and the answer comes back at once.
    let started = Instant::now();
    assert_eq!(status(&relay, &token).await, 401);
    assert!(started.elapsed() < Duration::from_secs(1));
    assert_eq!(relay.jwks_hits.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn last_good_keys_answer_during_failure_gap() {
    let relay = spawn(false).await;
    assert_eq!(status(&relay, &relay.mcp(USER_A)).await, 200);
    assert_eq!(relay.jwks_hits.load(Ordering::SeqCst), 1);

    // An unknown kid triggers a refetch, which fails. The cached key still works.
    relay.jwks_fail.store(true, Ordering::SeqCst);
    let stray = sign_with_kid(&claims(&relay.issuer, USER_A, true), "other-kid");
    assert_eq!(status(&relay, &stray).await, 401);
    assert_eq!(status(&relay, &relay.mcp(USER_A)).await, 200);
    assert_eq!(status(&relay, &stray).await, 401);
    assert_eq!(relay.jwks_hits.load(Ordering::SeqCst), 2);
}
