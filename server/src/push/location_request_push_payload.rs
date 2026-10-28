//! The APNs body for one locate request: an actionable alert the app answers in the background.

use serde_json::{Value, json};
use uuid::Uuid;

/// One locate request, as the push carries it.
pub struct Alert<'a> {
    pub request_id: Uuid,
    pub client_id: &'a str,
    /// Epoch ms when the relay stops waiting.
    pub expires_ms: u64,
}

/// `agent_name` is the OAuth client name; None says "An agent".
pub fn payload(alert: &Alert<'_>, agent_name: Option<&str>) -> Value {
    let text = match agent_name {
        Some(name) => format!("Your agent {name} wants to know your location"),
        None => "An agent wants to know your location".into(),
    };
    json!({
        "aps": {
            "alert": { "title": "Whereabot", "body": text },
            "sound": "default",
            "thread-id": "whereabot-locate",
            // The app registers this category with Reject, Allow and Always allow actions.
            "category": "LOCATION_REQUEST",
        },
        "request_id": alert.request_id,
        "client_id": alert.client_id,
        "expires_ms": alert.expires_ms,
    })
}
