mod common;

use common::{start_relay, supabase::agent_key_a};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

#[tokio::test]
async fn device_rejects_bad_token() {
    let addr = start_relay().await;
    for token in ["phone-x".to_string(), agent_key_a()] {
        let mut req = format!("ws://{addr}/v1/device")
            .into_client_request()
            .unwrap();
        req.headers_mut()
            .insert("authorization", format!("Bearer {token}").parse().unwrap());
        assert!(tokio_tungstenite::connect_async(req).await.is_err());
    }
    assert!(
        tokio_tungstenite::connect_async(format!("ws://{addr}/v1/device"))
            .await
            .is_err()
    );
}
