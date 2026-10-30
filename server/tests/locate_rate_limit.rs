mod common;

use common::{
    fake_phone::connect_silent_phone,
    fake_supabase::{USER_A, USER_B},
    locate_limit_fixtures::limits,
    locate_once, spawn_with,
};

#[tokio::test]
async fn locate_over_the_rate_gets_429_with_retry_after() {
    let relay = spawn_with(false, limits(1, 2, 10)).await;
    let http = reqwest::Client::new();
    let token = relay.mcp(USER_A);
    // Offline phones still get a push, so offline asks count too.
    for _ in 0..2 {
        assert_eq!(locate_once(&http, &relay.addr, &token).await.status(), 503);
    }
    let res = locate_once(&http, &relay.addr, &token).await;
    assert_eq!(res.status(), 429);
    let retry: u64 = res.headers()["retry-after"]
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!((1..=60).contains(&retry), "retry-after {retry}");
    // Another requester→owner pair keeps its own budget.
    let other = relay.mcp(USER_B);
    assert_eq!(locate_once(&http, &relay.addr, &other).await.status(), 503);
}

#[tokio::test]
async fn pending_requests_per_owner_are_capped() {
    let relay = spawn_with(false, limits(1, 10, 1)).await;
    let http = reqwest::Client::new();
    let token = relay.mcp(USER_A);
    let _silent = connect_silent_phone(&relay.addr, &relay.phone(USER_A)).await;
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let first = tokio::spawn({
        let (http, addr, token) = (http.clone(), relay.addr.clone(), token.clone());
        async move { locate_once(&http, &addr, &token).await.status() }
    });
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let res = locate_once(&http, &relay.addr, &token).await;
    assert_eq!(res.status(), 429);
    assert!(res.headers().contains_key("retry-after"));
    // The silent phone never answers: the first request times out (504), then the slot frees.
    assert_eq!(first.await.unwrap(), 504);
    assert_eq!(locate_once(&http, &relay.addr, &token).await.status(), 504);
}
