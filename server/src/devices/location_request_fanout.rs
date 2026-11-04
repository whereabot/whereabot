use std::sync::Arc;

use uuid::Uuid;

use super::device_connection_registry::{Registry, Tx};
use crate::{locate_request::LocateRequest, relay_wire_protocol::ToDevice};

/// One locate request sent to every device of a user. First answer wins; the rest get `cancel`.
pub struct Fanout {
    user: Uuid,
    request: Arc<LocateRequest>,
    sent: Vec<Tx>,
}

impl Fanout {
    pub fn new(user: Uuid, request: Arc<LocateRequest>) -> Self {
        Self {
            user,
            request,
            sent: Vec::new(),
        }
    }

    /// Pings each connected device that has not seen this request yet (new or reconnected).
    pub async fn send_new(&mut self, devices: &Registry) {
        for (_, device) in devices.senders(self.user).await {
            if self.sent.iter().any(|s| s.same_channel(&device)) {
                continue;
            }
            let ping = ToDevice::Locate {
                id: self.request.id,
                client_id: self.request.client_id.clone(),
                expires_ms: self.request.expires_ms,
                requester: self.request.requester.clone(),
                reason: self.request.reason.clone(),
            };
            if device.send(ping).await.is_ok() {
                self.sent.push(device);
            }
        }
    }

    /// True once any device got the ping.
    pub fn reached(&self) -> bool {
        !self.sent.is_empty()
    }

    /// Closes the prompt on every pinged device except `winner` (nil = none answered).
    pub async fn cancel_others(&self, devices: &Registry, winner: Uuid) {
        for (key, device) in devices.senders(self.user).await {
            if key != winner && self.sent.iter().any(|s| s.same_channel(&device)) {
                let _ = device.try_send(ToDevice::Cancel {
                    id: self.request.id,
                });
            }
        }
    }
}
