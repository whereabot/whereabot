use std::time::Duration;

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use reqwest::header::HeaderValue;
use web_push_native::jwt_simple::algorithms::ES256KeyPair;

use super::web_push_subscription_store::Subscription;
use crate::push::apns_sender::Sent;

/// VAPID `sub`: who push services contact about this sender.
const CONTACT: &str = "https://whereabot.io";

pub struct WebPush {
    http: reqwest::Client,
    key: ES256KeyPair,
}

impl WebPush {
    /// `private_key`: base64url of the raw 32-byte P-256 scalar (`VAPID_PRIVATE_KEY`).
    pub fn new(private_key: &str) -> Option<Self> {
        let raw = URL_SAFE_NO_PAD.decode(private_key.trim().trim_end_matches('='));
        let key = raw.ok().and_then(|raw| ES256KeyPair::from_bytes(&raw).ok());
        if key.is_none() {
            tracing::warn!("web push off: VAPID_PRIVATE_KEY is not a base64url P-256 key");
        }
        Some(Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .ok()?,
            key: key?,
        })
    }

    /// One encrypted push. `ttl` is how long the push service may hold it (also the VAPID JWT life).
    pub async fn send(&self, sub: &Subscription, body: &[u8], ttl: Duration) -> Sent {
        let Some(builder) = sub.builder() else {
            tracing::warn!("web push skipped: invalid subscription or unknown push service");
            return Sent::Gone;
        };
        let built = builder
            .with_valid_duration(ttl)
            .with_vapid(&self.key, CONTACT)
            .build(body);
        let Some(mut req) = built.ok().and_then(|r| reqwest::Request::try_from(r).ok()) else {
            tracing::warn!("web push encryption failed");
            return Sent::Failed;
        };
        req.headers_mut()
            .insert("urgency", HeaderValue::from_static("high"));
        let status = match self.http.execute(req).await {
            Ok(res) => res.status(),
            Err(err) => {
                tracing::warn!(error = %err.without_url(), "web push request failed");
                return Sent::Failed;
            }
        };
        match status.as_u16() {
            200..=299 => Sent::Ok,
            // The browser unsubscribed or the subscription expired.
            404 | 410 => Sent::Gone,
            _ => {
                tracing::warn!(%status, "push service rejected web push");
                Sent::Failed
            }
        }
    }
}
