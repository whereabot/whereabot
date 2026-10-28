mod common;

use common::{
    Relay,
    jwt::{self, claims, sign_hs256, sign_none, sign_with_kid},
    spawn,
    supabase::USER_A,
};
use serde_json::{Value, json};

const ACCEPT: &str = "application/json, text/event-stream";
const EVIL_ISSUER: &str = "https://evil.example/auth/v1";

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

/// MCP claims with one field replaced.
fn with(field: &str, value: Value, relay: &Relay) -> Value {
    let mut claims = claims(&relay.issuer, USER_A, true);
    claims[field] = value;
    claims
}

#[tokio::test]
async fn valid_token_and_leeway_accepted() {
    let relay = spawn(false).await;
    assert_eq!(status(&relay, &relay.mcp(USER_A)).await, 200);
    // 30 s leeway: a token that expired 10 s ago still passes.
    let recent = with("exp", json!(jwt::now_secs() - 10), &relay);
    assert_eq!(status(&relay, &jwt::sign(&recent)).await, 200);
}

#[tokio::test]
async fn expired_token_rejected() {
    let relay = spawn(false).await;
    let old = with("exp", json!(jwt::now_secs() - 120), &relay);
    assert_eq!(status(&relay, &jwt::sign(&old)).await, 401);
}

#[tokio::test]
async fn wrong_audience_rejected() {
    let relay = spawn(false).await;
    let token = jwt::sign(&with("aud", json!("anon"), &relay));
    assert_eq!(status(&relay, &token).await, 401);
}

#[tokio::test]
async fn wrong_issuer_rejected() {
    let relay = spawn(false).await;
    let token = jwt::sign(&with("iss", json!(EVIL_ISSUER), &relay));
    assert_eq!(status(&relay, &token).await, 401);
}

#[tokio::test]
async fn non_uuid_subject_rejected() {
    let relay = spawn(false).await;
    let token = jwt::sign(&with("sub", json!("not-a-uuid"), &relay));
    assert_eq!(status(&relay, &token).await, 401);
}

#[tokio::test]
async fn alg_none_rejected() {
    let relay = spawn(false).await;
    let token = sign_none(&claims(&relay.issuer, USER_A, true));
    assert_eq!(status(&relay, &token).await, 401);
}

#[tokio::test]
async fn hs256_rejected() {
    let relay = spawn(false).await;
    let token = sign_hs256(&claims(&relay.issuer, USER_A, true));
    assert_eq!(status(&relay, &token).await, 401);
}

#[tokio::test]
async fn unknown_kid_refetches_once_per_minute() {
    let relay = spawn(false).await;
    assert_eq!(status(&relay, &relay.mcp(USER_A)).await, 200);
    assert_eq!(relay.jwks_hits.load(std::sync::atomic::Ordering::SeqCst), 1);

    let stray = sign_with_kid(&claims(&relay.issuer, USER_A, true), "other-kid");
    assert_eq!(status(&relay, &stray).await, 401);
    assert_eq!(status(&relay, &stray).await, 401);
    // One refetch for the unknown kid. The second miss falls inside the 60 s gap.
    assert_eq!(relay.jwks_hits.load(std::sync::atomic::Ordering::SeqCst), 2);
}
