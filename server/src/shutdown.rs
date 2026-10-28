use std::sync::Arc;

use crate::{AppState, protocol::ToDevice};

/// Handle for graceful shutdown.
pub struct Shutdown(pub(crate) Arc<AppState>);

impl Shutdown {
    /// Sends Close 1012 "restart" to every device socket so apps reconnect at once.
    pub async fn close_devices(&self) {
        for device in self.0.devices.lock().await.values() {
            let _ = device.try_send(ToDevice::Close {
                code: 1012,
                reason: "restart",
            });
        }
    }
}
