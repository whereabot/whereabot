//! APNs to every token of the user once.

use serde_json::Value;

use super::{
    Push,
    apns_sender::{Kind, Sent},
    location_request_push_payload::Alert,
};

impl Push {
    /// Drops tokens APNs says are gone.
    pub(super) async fn send_apns(
        &self,
        bearer: &str,
        alert: &Alert<'_>,
        body: &Value,
        kind: Kind,
    ) -> usize {
        let Some(apns) = &self.apns else { return 0 };
        let tokens = self.tokens.list(bearer, alert.owner).await;
        if tokens.is_empty() {
            tracing::info!(request_id = %alert.request_id, "push skipped: no push token for this user");
            return 0;
        }
        let mut accepted = 0;
        let total = tokens.len();
        for token in tokens {
            match apns.send(&token, body, kind, alert.expires_ms / 1000).await {
                Sent::Ok => accepted += 1,
                // shortcut: a requester's JWT cannot delete the owner's token; the owner's own next request does.
                Sent::Gone if alert.owner.is_none() => self.tokens.delete(bearer, &token).await,
                Sent::Gone => {}
                Sent::Failed => {}
            }
        }
        tracing::info!(request_id = %alert.request_id, total, accepted, "push sent");
        accepted
    }
}
