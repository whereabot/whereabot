//! The APNs bodies for one locate request: the consent alert, and for always-allowed agents
//! a silent wake-up plus a "checking" alert when the silent one gets no answer.

use serde_json::{Value, json};
use uuid::Uuid;

use crate::relay_wire_protocol::Requester;

/// One locate request, as the push carries it.
pub struct Alert<'a> {
    pub request_id: Uuid,
    pub client_id: &'a str,
    /// Epoch ms when the relay stops waiting.
    pub expires_ms: u64,
    /// The phones' owner when someone else's agent asks; their tokens come from an RPC.
    pub owner: Option<Uuid>,
    /// Who asks, when it is not the owner's own agent.
    pub requester: Option<&'a Requester>,
}

/// Who asks, as the alert names them: "Your agent X", "Ann's agent X", "Ann's agent".
fn asker(alert: &Alert<'_>, agent_name: Option<&str>) -> String {
    match (alert.requester, agent_name) {
        (Some(who), Some(agent)) => format!("{}'s agent {agent}", who.name),
        (Some(who), None) => format!("{}'s agent", who.name),
        (None, Some(agent)) => format!("Your agent {agent}"),
        (None, None) => "An agent".into(),
    }
}

/// `agent_name` is the OAuth client name; None says "An agent".
pub fn payload(alert: &Alert<'_>, agent_name: Option<&str>) -> Value {
    let text = format!("{} wants to know your location", asker(alert, agent_name));
    // The app registers this category with Reject, Allow and Always allow actions.
    with_alert(alert, agent_name, text, "LOCATION_REQUEST")
}

/// Fallback for an always-allowed agent: no question, one Share action (category LOCATION_AUTO).
pub fn checking(alert: &Alert<'_>, agent_name: Option<&str>) -> Value {
    let who = match (alert.requester, agent_name) {
        (None, name) => name.unwrap_or("Your agent").to_owned(),
        _ => asker(alert, agent_name),
    };
    with_alert(
        alert,
        agent_name,
        format!("{who} is checking your location"),
        "LOCATION_AUTO",
    )
}

/// No alert, sound or badge: iOS wakes the app in the background to answer on its own.
pub fn silent(alert: &Alert<'_>) -> Value {
    json!({
        "aps": { "content-available": 1 },
        "request_id": alert.request_id,
        "client_id": alert.client_id,
        "expires_ms": alert.expires_ms,
        "requester": alert.requester,
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
        // Always allow on a shared request is stored for this requester only.
        "requester": alert.requester,
    })
}
