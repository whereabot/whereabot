mod common;

use common::{
    fake_supabase::{USER_A, USER_B},
    locate_limit_fixtures::limits,
    locate_once,
    mcp_client::{call_location, rpc},
    spawn_with,
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
async fn mcp_tool_over_the_rate_returns_error_with_wait() {
    let relay = spawn_with(false, limits(1, 1, 10)).await;
    let http = reqwest::Client::new();
    let token = relay.mcp(USER_A);
    let first = rpc(&http, &relay.addr, &token, call_location(1)).await;
    assert_eq!(first["result"]["isError"], true);
    assert!(
        !first["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Too many")
    );
    let limited = rpc(&http, &relay.addr, &token, call_location(2)).await;
    assert_eq!(limited["result"]["isError"], true);
    let text = limited["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        text.starts_with("Too many location requests. Wait "),
        "{text}"
    );
    assert!(text.ends_with(" seconds before asking again."), "{text}");
}
