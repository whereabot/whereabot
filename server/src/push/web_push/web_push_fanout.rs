//! Web Push next to APNs: the same alert text to every browser/PWA install of the user.
//! Only alerts: a service worker cannot read the location, so a silent wake-up is useless there.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use crate::push::{Push, apns_sender::Sent, location_request_push_payload::Alert};

/// Push services must hold the push at least this long, and the VAPID JWT must not expire at once.
const MIN_TTL: Duration = Duration::from_secs(60);
/// FCM rejects VAPID JWTs that live longer than 24 h.
const MAX_TTL: Duration = Duration::from_secs(12 * 60 * 60);

/// What `sw.js` shows: the APNs alert's title and body, plus the request for the click.
pub fn web_body(alert: &Alert<'_>, apns_body: &Value) -> Vec<u8> {
    let shown = &apns_body["aps"]["alert"];
    json!({
        "title": shown["title"],
        "body": shown["body"],
        "request_id": alert.request_id,
        "expires_ms": alert.expires_ms,
    })
    .to_string()
    .into_bytes()
}

fn ttl(expires_ms: u64) -> Duration {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    Duration::from_millis(expires_ms)
        .saturating_sub(now)
        .clamp(MIN_TTL, MAX_TTL)
}

impl Push {
    /// Returns how many pushes the push services accepted; drops subscriptions that are gone.
    pub(in crate::push) async fn send_web(
        &self,
        bearer: &str,
        alert: &Alert<'_>,
        apns_body: &Value,
    ) -> usize {
        let Some(web) = &self.web else { return 0 };
        let subs = self.subscriptions.list(bearer, alert.owner).await;
        if subs.is_empty() {
            return 0;
        }
        let (body, ttl) = (web_body(alert, apns_body), ttl(alert.expires_ms));
        let mut accepted = 0;
        for sub in &subs {
            match (web.send(sub, &body, ttl).await, sub.id) {
                (Sent::Ok, _) => accepted += 1,
                // Only own rows carry an id: a requester's JWT cannot delete the owner's.
                (Sent::Gone, Some(id)) => self.subscriptions.delete(bearer, id).await,
                _ => {}
            }
        }
        tracing::info!(request_id = %alert.request_id, total = subs.len(), accepted, "web push sent");
        accepted
    }
}
