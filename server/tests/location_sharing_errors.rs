mod common;

use common::{
    fake_location_shares::USER_C,
    fake_supabase::USER_B,
    sharing_mcp_client::{call, share},
    start_relay,
};
use serde_json::{Value, json};

#[tokio::test]
async fn unknown_revoked_and_ambiguous_people_get_error_text() {
    let relay = start_relay().await;
    relay.shares.lock().unwrap().extend([
        share(USER_B, "Dana", "dana.b@example.com", false),
        share(USER_C, "Dana", "dana.c@example.com", false),
        share(USER_B, "Ex", "ex@example.com", true),
    ]);
    let not_shared = |who: &str| {
        format!(
            "Nobody named {who} shares their location with you. Ask them to share it in the Whereabot app."
        )
    };
    for who in ["Stranger", "Ex"] {
        let (error, text) = call(&relay, "get_phone_location", json!({ "person": who })).await;
        assert!(error);
        assert_eq!(text, not_shared(who));
    }
    let (error, text) = call(&relay, "get_phone_location", json!({ "person": "dana" })).await;
    assert!(error);
    assert!(
        text.contains("Dana (dana.b@example.com), Dana (dana.c@example.com)"),
        "{text}"
    );
}

#[tokio::test]
async fn list_tool_returns_active_shares_only() {
    let relay = start_relay().await;
    relay.shares.lock().unwrap().extend([
        share(USER_B, "Brother", "bro@example.com", false),
        share(USER_C, "Ex", "ex@example.com", true),
    ]);
    let (error, text) = call(&relay, "list_people_sharing_with_me", json!({})).await;
    assert!(!error);
    let people: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        people,
        json!([{ "display_name": "Brother", "email": "bro@example.com" }])
    );
}
