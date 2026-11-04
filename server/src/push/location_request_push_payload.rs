//! The APNs bodies for one locate request: the consent alert, and for always-allowed agents
//! a silent wake-up plus a "checking" alert when the silent one gets no answer.

use serde_json::{Value, json};
use uuid::Uuid;

use crate::locate_request::LocateRequest;

/// One locate request as the push shows it, plus the owner when someone else's agent asks.
pub struct Alert<'a> {
    pub request: &'a LocateRequest,
    /// The phones' owner when someone else's agent asks; their tokens come from an RPC.
    pub owner: Option<Uuid>,
}

/// Who asks, as the alert names them: "Your agent X", "Ann's agent X", "Ann's agent".
fn asker(alert: &Alert<'_>, agent_name: Option<&str>) -> String {
    match (alert.request.requester.as_ref(), agent_name) {
        (Some(who), Some(agent)) => format!("{}'s agent {agent}", who.name),
        (Some(who), None) => format!("{}'s agent", who.name),
        (None, Some(agent)) => format!("Your agent {agent}"),
        (None, None) => "An agent".into(),
    }
}

/// `agent_name` is the OAuth client name; None says "An agent".
pub fn payload(alert: &Alert<'_>, agent_name: Option<&str>) -> Value {
    let mut text = format!("{} wants to know your location", asker(alert, agent_name));
    if let Some(reason) = &alert.request.reason {
        text.push_str(&format!("\nReason: {reason}"));
    }
    // The app registers this category with Reject, Allow and Always allow actions.
    with_alert(alert, agent_name, text, "LOCATION_REQUEST")
}

/// Fallback for an always-allowed agent: no question, one Share action (category LOCATION_AUTO).
pub fn checking(alert: &Alert<'_>, agent_name: Option<&str>) -> Value {
    let who = match (alert.request.requester.as_ref(), agent_name) {
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
    let request = alert.request;
    json!({
        "aps": { "content-available": 1 },
        "request_id": request.id,
        "client_id": request.client_id,
        "expires_ms": request.expires_ms,
        "requester": request.requester,
        "reason": request.reason,
    })
}

fn with_alert(alert: &Alert<'_>, agent_name: Option<&str>, text: String, category: &str) -> Value {
    let request = alert.request;
    json!({
        "aps": {
            "alert": { "title": "Whereabot", "body": text },
            "sound": "default",
            "thread-id": "whereabot-locate",
            "category": category,
        },
        "request_id": request.id,
        "client_id": request.client_id,
        // Lets the app store the name when the user taps Always allow on the lock screen.
        "agent_name": agent_name,
        "expires_ms": request.expires_ms,
        // Always allow on a shared request is stored for this requester only.
        "requester": request.requester,
        "reason": request.reason,
    })
}

#[cfg(test)]
mod tests {
    use super::payload;
    use crate::push::push_wake_tests::{alert, request};
    use uuid::Uuid;

    #[test]
    fn the_alert_shows_the_reason_under_the_question() {
        let mut req = request(Uuid::nil(), "claude-code");
        req.reason = Some("To find coffee near you".into());
        let body = payload(&alert(&req), Some("Claude"));
        assert_eq!(
            body["aps"]["alert"]["body"],
            "Your agent Claude wants to know your location\nReason: To find coffee near you"
        );
        assert_eq!(body["reason"], "To find coffee near you");
    }
}
