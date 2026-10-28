mod common;

use common::start_relay;

#[tokio::test]
async fn healthz_is_ok_without_auth() {
    let addr = start_relay().await;
    let res = reqwest::Client::new()
        .get(format!("http://{addr}/healthz"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.text().await.unwrap(), "ok");
}
