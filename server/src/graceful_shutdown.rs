use std::sync::Arc;

use crate::AppState;

/// Handle for graceful shutdown.
pub struct Shutdown(pub(crate) Arc<AppState>);

impl Shutdown {
    /// Sends Close 1012 "restart" to every device socket so apps reconnect at once.
    pub async fn close_devices(&self) {
        self.0.devices.close_all(1012, "restart").await;
    }
}
