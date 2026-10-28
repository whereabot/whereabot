mod common;

use common::{
    locate_settled, mcp::call_settled, scripted::connect_scripted, start_relay, supabase::USER_A,
};
use serde_json::{Value, json};

const CASES: [(&str, &str, u16, &str); 5] = [
    (
        "denied",
        "no_answer",
        408,
        "The user did not respond on their phone. Ask them before trying again.",
    ),
    (
        "unavailable",
        "permission",
        424,
        "Location access is off for Whereabot on the user's phone. Ask them to allow it in Settings.",
    ),
    (
        "unavailable",
        "no_fix",
        424,
        "The phone could not get a location fix. Try again in a moment.",
    ),
    (
        "unavailable",
        "error",
        424,
        "The phone could not get a location fix. Try again in a moment.",
    ),
    (
        "denied",
        "user",
        403,
        "The user declined to share their location.",
    ),
];

fn answer(kind: &'static str, reason: &'static str) -> impl Fn(&Value) -> Option<Value> {
    move |ping| Some(json!({ "type": kind, "id": ping["id"], "reason": reason }))
}

#[tokio::test]
async fn http_maps_each_reason_to_status_and_text() {
    for (kind, reason, status, text) in CASES {
        let relay = start_relay().await;
        let http = reqwest::Client::new();
        let phone = relay.phone(USER_A);
        connect_scripted(&relay.addr, &phone, None, answer(kind, reason)).await;
        let res = locate_settled(&http, &relay.addr, &relay.mcp(USER_A)).await;
        assert_eq!(res.status(), status, "{kind} {reason}");
        assert_eq!(res.text().await.unwrap(), text);
    }
}

#[tokio::test]
async fn mcp_maps_each_reason_to_text() {
    for (kind, reason, _, text) in CASES {
        let relay = start_relay().await;
        let http = reqwest::Client::new();
        let phone = relay.phone(USER_A);
        connect_scripted(&relay.addr, &phone, None, answer(kind, reason)).await;
        let res = call_settled(&http, &relay.addr, &relay.mcp(USER_A), 1).await;
        assert_eq!(res["result"]["isError"], true);
        assert_eq!(res["result"]["content"][0]["text"], text);
    }
}

#[tokio::test]
async fn denied_without_reason_means_user() {
    let relay = start_relay().await;
    let http = reqwest::Client::new();
    let reply = |ping: &Value| Some(json!({ "type": "denied", "id": ping["id"] }));
    connect_scripted(&relay.addr, &relay.phone(USER_A), None, reply).await;
    let res = locate_settled(&http, &relay.addr, &relay.mcp(USER_A)).await;
    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn offline_text_names_the_account_email() {
    let relay = start_relay().await;
    let http = reqwest::Client::new();
    let token = relay.mcp_email(USER_A, "me@example.test");
    let res = common::locate_once(&http, &relay.addr, &token).await;
    assert_eq!(res.status(), 503);
    assert_eq!(
        res.text().await.unwrap(),
        "The user's phone is not connected. Ask the user to open the Whereabot app, signed in as me@example.test and keep it open, then retry."
    );
}
