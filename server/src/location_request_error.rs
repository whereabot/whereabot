use axum::http::StatusCode;

use crate::relay_wire_protocol::{DeniedReason, UnavailableReason};

/// Why a locate request ended without a fix, with the agent-facing text and HTTP status.
pub enum LocateError {
    Offline {
        email: Option<String>,
    },
    Timeout,
    Denied(DeniedReason),
    /// A person who shares with the agent's user declined; carries their name.
    DeniedBy(String),
    Unavailable(UnavailableReason),
    /// `person` names nobody with an active share to the agent's user.
    NotShared(String),
    /// `person` names more than one sharer; `matches` labels each.
    Ambiguous {
        person: String,
        matches: Vec<String>,
    },
}

impl LocateError {
    pub fn status(&self) -> StatusCode {
        match self {
            Self::Offline { .. } => StatusCode::SERVICE_UNAVAILABLE,
            Self::Timeout => StatusCode::GATEWAY_TIMEOUT,
            Self::Denied(DeniedReason::NoAnswer) => StatusCode::REQUEST_TIMEOUT,
            Self::Denied(_) | Self::DeniedBy(_) => StatusCode::FORBIDDEN,
            Self::Unavailable(_) => StatusCode::FAILED_DEPENDENCY,
            Self::NotShared(_) => StatusCode::NOT_FOUND,
            Self::Ambiguous { .. } => StatusCode::CONFLICT,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Offline { email } => {
                let account = email
                    .as_ref()
                    .map_or(String::new(), |email| format!(", signed in as {email}"));
                format!(
                    "The user's phone is not connected. Ask the user to open the Whereabot app{account} and keep it open, then retry."
                )
            }
            Self::Timeout => {
                "The phone did not answer in time. Ask the user to open Whereabot and retry.".into()
            }
            Self::Denied(DeniedReason::NoAnswer) => {
                "The user did not respond on their phone. Ask them before trying again.".into()
            }
            Self::Denied(_) => "The user declined to share their location.".into(),
            Self::DeniedBy(name) => format!("{name} declined to share their location."),
            Self::Unavailable(UnavailableReason::Permission) => {
                "Location access is off for Whereabot on the user's phone. Ask them to allow it in Settings.".into()
            }
            Self::Unavailable(_) => {
                "The phone could not get a location fix. Try again in a moment.".into()
            }
            Self::NotShared(person) => format!(
                "Nobody named {person} shares their location with you. Ask them to share it in the Whereabot app."
            ),
            Self::Ambiguous { person, matches } => format!(
                "More than one person named {person} shares their location with you: {}. Ask again with their email.",
                matches.join(", ")
            ),
        }
    }
}
