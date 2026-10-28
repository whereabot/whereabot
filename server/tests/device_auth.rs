mod common;

use common::{
    agent_rpc::agent_key_a,
    jwt::{claims, sign_none},
    spawn, start_relay,
    supabase::USER_A,
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
async fn device_rejects_agent_key_even_when_fallback_on() {
    let relay = spawn(true).await;
    assert!(!handshake(&relay.addr, Some(&agent_key_a())).await);
}
