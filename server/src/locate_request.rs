//! One locate request, shared by every layer. The owner's phones live in `Target`, not here.

use uuid::Uuid;

use crate::relay_wire_protocol::Requester;

/// Add a field here and every layer gets it: fanout, wake push and alert.
pub struct LocateRequest {
    pub id: Uuid,
    pub client_id: String,
    /// Epoch ms when the relay stops waiting.
    pub expires_ms: u64,
    /// Set when someone else's agent asks.
    pub requester: Option<Requester>,
    /// Why the agent asks, already cleaned.
    pub reason: Option<String>,
}
