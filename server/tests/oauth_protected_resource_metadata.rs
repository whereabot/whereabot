mod common;

use common::{CHALLENGE, start_relay};
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::{Error, client::IntoClientRequest};

#[tokio::test]
async fn metadata_matches_c1_at_both_paths() {
    let relay = start_relay().await;
    let expected = json!({
        "resource": "https://relay.test/mcp",
        "authorization_servers": [relay.issuer],
        "bearer_methods_supported": ["header"],
    });
    for path in [
        "/.well-known/oauth-protected-resource",
        "/.well-known/oauth-protected-resource/mcp",
    ] {
        let res = reqwest::get(format!("http://{}{path}", relay.addr))
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body: Value = res.json().await.unwrap();
        assert_eq!(body, expected, "{path}");
    }
}

#[tokio::test]
async fn device_401_carries_challenge() {
    let relay = start_relay().await;
    let req = format!("ws://{}/v1/device", relay.addr)
        .into_client_request()
        .unwrap();
    match tokio_tungstenite::connect_async(req).await {
        Err(Error::Http(res)) => {
            assert_eq!(res.status(), 401);
            assert_eq!(res.headers()["www-authenticate"], CHALLENGE);
        }
        other => panic!("expected a 401 handshake, got {:?}", other.err()),
    }
}
