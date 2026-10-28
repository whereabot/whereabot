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

/// Why the user did not share. Unknown reasons count as `User`.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeniedReason {
    NoAnswer,
    #[default]
    #[serde(other)]
    User,
}

/// Why the phone could not look. Unknown reasons count as `Error`.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnavailableReason {
    Permission,
    NoFix,
    #[serde(other)]
    Error,
}

/// How the phone settled one locate request.
pub enum Answer {
    Fix(Location),
    Denied(DeniedReason),
    Unavailable(UnavailableReason),
}

/// Server -> device. `client_id` names the asking agent; the app maps it to a display name.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToDevice {
    /// `expires_ms`: epoch ms when the relay stops waiting.
    Locate {
        id: Uuid,
        client_id: String,
        expires_ms: u64,
    },
    /// Another device answered `id`: close the prompt.
    Cancel { id: Uuid },
    /// Not a message: the socket loop sends a Close frame with this code and reason.
    #[serde(skip)]
    Close { code: u16, reason: &'static str },
}

/// Device -> server.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FromDevice {
    Hello {
        app_version: Option<String>,
        device_id: Option<Uuid>,
        device_name: Option<String>,
        platform: Option<String>,
    },
    Location(Location),
    Denied {
        id: Uuid,
        #[serde(default)]
        reason: DeniedReason,
    },
    Unavailable {
        id: Uuid,
        reason: UnavailableReason,
    },
    /// App diagnostics (no location data); the relay only logs them.
    Log {
        msg: String,
    },
}
