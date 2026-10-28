use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Fix sent by the phone in answer to a `locate` ping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_m: Option<f64>,
    pub captured_at_ms: u64,
}

/// How the phone settled one locate request.
pub enum Answer {
    Fix(Location),
    Denied,
}

/// Server -> device. `client_id` names the asking agent; the app maps it to a display name.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToDevice {
    Locate { id: Uuid, client_id: String },
}

/// Device -> server.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FromDevice {
    Location(Location),
    Denied { id: Uuid },
}
