//! The APNs bodies for one locate request: the consent alert, and for always-allowed agents
//! a silent wake-up plus a "checking" alert when the silent one gets no answer.

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
    // The app registers this category with Reject, Allow and Always allow actions.
    with_alert(alert, agent_name, text, "LOCATION_REQUEST")
}

/// Fallback for an always-allowed agent: no question, one Share action (category LOCATION_AUTO).
pub fn checking(alert: &Alert<'_>, agent_name: Option<&str>) -> Value {
    let text = format!(
        "{} is checking your location",
        agent_name.unwrap_or("Your agent")
    );
    with_alert(alert, agent_name, text, "LOCATION_AUTO")
}

/// No alert, sound or badge: iOS wakes the app in the background to answer on its own.
pub fn silent(alert: &Alert<'_>) -> Value {
    json!({
        "aps": { "content-available": 1 },
        "request_id": alert.request_id,
        "client_id": alert.client_id,
        "expires_ms": alert.expires_ms,
    })
}

fn with_alert(alert: &Alert<'_>, agent_name: Option<&str>, text: String, category: &str) -> Value {
    json!({
        "aps": {
            "alert": { "title": "Whereabot", "body": text },
            "sound": "default",
            "thread-id": "whereabot-locate",
            "category": category,
        },
        "request_id": alert.request_id,
        "client_id": alert.client_id,
        // Lets the app store the name when the user taps Always allow on the lock screen.
        "agent_name": agent_name,
        "expires_ms": alert.expires_ms,
    })
}
