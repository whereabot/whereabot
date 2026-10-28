//! Which push a waiting request gets. An always-allowed agent never "wants to know": the app
//! is woken silently to answer on its own, and only if it stays quiet does the user see
//! "<Agent> is checking your location" with one Share action.

use std::time::Duration;

use super::{
    Push,
    apns_sender::Kind,
    location_request_push_payload::{Alert, checking, silent},
};

impl Push {
    /// The wake-up after `delay`. Abort the task to cancel it (the request was answered).
    pub async fn wake_after(&self, delay: Duration, bearer: &str, alert: &Alert<'_>) {
        tokio::time::sleep(delay).await;
        if !self
            .always
            .contains(bearer, alert.client_id, alert.owner)
            .await
        {
            self.wake(bearer, alert).await;
            return;
        }
        self.send_all(bearer, alert, &silent(alert), Kind::Background)
            .await;
        // iOS skips silent pushes for force-quit apps and in Low Power Mode, and throttles them.
        tokio::time::sleep(self.silent_window).await;
        let name = self.names.name(bearer, alert.client_id).await;
        let body = checking(alert, name.as_deref());
        self.send_all(bearer, alert, &body, Kind::Alert).await;
    }
}
