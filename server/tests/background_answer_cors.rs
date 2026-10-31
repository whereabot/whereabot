mod common;

use common::start_relay;

#[tokio::test]
async fn webview_preflight_is_allowed() {
    let relay = start_relay().await;
    let res = reqwest::Client::new()
        .request(
            reqwest::Method::OPTIONS,
            format!("http://{}/v1/device/answer", relay.addr),
        )
        .header("Origin", "tauri://localhost")
        .header("Access-Control-Request-Method", "POST")
        .header(
            "Access-Control-Request-Headers",
            "authorization,content-type",
        )
        .send()
        .await
        .unwrap();
    assert!(res.status().is_success());
    let allow = |name: &str| res.headers()[name].to_str().unwrap().to_lowercase();
    assert_eq!(allow("access-control-allow-origin"), "*");
    assert!(allow("access-control-allow-headers").contains("authorization"));
    assert!(allow("access-control-allow-methods").contains("post"));
}
