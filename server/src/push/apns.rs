use std::time::Duration;

use serde_json::json;
use uuid::Uuid;

use super::{config::PushConfig, jwt::ProviderJwt};

pub enum Sent {
    Ok,
    /// Unregistered or BadDeviceToken: the app was removed or the token rotated.
    Gone,
    Failed,
}

pub struct Apns {
    http: reqwest::Client,
    url: String,
    topic: String,
    jwt: ProviderJwt,
}

impl Apns {
    pub fn new(config: &PushConfig) -> Option<Self> {
        let jwt = ProviderJwt::new(&config.key_pem, &config.key_id, &config.team_id);
        if jwt.is_none() {
            tracing::warn!("push off: APNS_KEY is not a valid PEM key");
        }
        Some(Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .ok()?,
            url: config.apns_url.clone(),
            topic: config.topic.clone(),
            jwt: jwt?,
        })
    }

    /// One alert push. `expires_secs` is epoch seconds: APNs drops it after that.
    pub async fn send(&self, device_token: &str, request_id: Uuid, expires_secs: u64) -> Sent {
        let Some(jwt) = self.jwt.get().await else {
            return Sent::Failed;
        };
        let body = json!({
            "aps": {
                "alert": { "title": "Whereabot", "body": "Your agent wants to know your location" },
                "sound": "default",
                "thread-id": "whereabot-locate",
            },
            "request_id": request_id,
        });
        let res = self
            .http
            .post(format!("{}/3/device/{device_token}", self.url))
            .bearer_auth(jwt)
            .header("apns-topic", &self.topic)
            .header("apns-push-type", "alert")
            .header("apns-priority", "10")
            .header("apns-expiration", expires_secs.to_string())
            .json(&body)
            .send()
            .await;
        let res = match res {
            Ok(res) => res,
            Err(err) => {
                tracing::warn!(error = %err.without_url(), "apns request failed");
                return Sent::Failed;
            }
        };
        let status = res.status();
        if status.is_success() {
            return Sent::Ok;
        }
        let reason = res
            .json::<serde_json::Value>()
            .await
            .ok()
            .and_then(|v| v["reason"].as_str().map(String::from));
        tracing::warn!(%status, reason = reason.as_deref().unwrap_or("?"), "apns rejected push");
        let gone = status.as_u16() == 410 || reason.as_deref() == Some("BadDeviceToken");
        if gone { Sent::Gone } else { Sent::Failed }
    }
}
