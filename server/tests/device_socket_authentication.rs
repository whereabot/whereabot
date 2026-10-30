mod common;

use common::{
    agent_key_rpc_fake::agent_key_a,
    fake_supabase::USER_A,
    spawn, start_relay,
    test_token_signer::{claims, sign_none},
};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

async fn handshake(addr: &str, token: Option<&str>) -> bool {
    let mut req = format!("ws://{addr}/v1/device")
        .into_client_request()
        .unwrap();
    if let Some(token) = token {
        req.headers_mut()
            .insert("authorization", format!("Bearer {token}").parse().unwrap());
    }
    tokio_tungstenite::connect_async(req).await.is_ok()
}

#[tokio::test]
async fn device_accepts_phone_token() {
    let relay = start_relay().await;
    assert!(handshake(&relay.addr, Some(&relay.phone(USER_A))).await);
}

#[tokio::test]
async fn device_rejects_bad_token() {
    let relay = start_relay().await;
    let unsigned = sign_none(&claims(&relay.issuer, USER_A, false));
    for token in ["phone-x", unsigned.as_str()] {
        assert!(!handshake(&relay.addr, Some(token)).await, "{token}");
    }
    assert!(!handshake(&relay.addr, Some(&agent_key_a())).await);
    assert!(!handshake(&relay.addr, None).await);
}

#[tokio::test]
async fn device_rejects_mcp_agent_token() {
    let relay = start_relay().await;
    assert!(!handshake(&relay.addr, Some(&relay.mcp(USER_A))).await);
}

#[tokio::test]
async fn device_rejects_agent_key_even_when_fallback_on() {
    let relay = spawn(true).await;
    assert!(!handshake(&relay.addr, Some(&agent_key_a())).await);
}

/// The browser app (PWA) cannot set Authorization: it sends the token as a subprotocol.
#[tokio::test]
async fn device_accepts_token_in_subprotocol() {
    let relay = start_relay().await;
    let mut req = format!("ws://{}/v1/device", relay.addr)
        .into_client_request()
        .unwrap();
    let offer = format!("whereabot.bearer, {}", relay.phone(USER_A));
    req.headers_mut()
        .insert("sec-websocket-protocol", offer.parse().unwrap());
    let (_, res) = tokio_tungstenite::connect_async(req)
        .await
        .expect("handshake");
    assert_eq!(res.headers()["sec-websocket-protocol"], "whereabot.bearer");
    let mut bad = format!("ws://{}/v1/device", relay.addr)
        .into_client_request()
        .unwrap();
    bad.headers_mut().insert(
        "sec-websocket-protocol",
        "whereabot.bearer, phone-x".parse().unwrap(),
    );
    assert!(tokio_tungstenite::connect_async(bad).await.is_err());
}
