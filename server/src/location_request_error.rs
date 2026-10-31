use std::time::Duration;

use axum::http::StatusCode;

use crate::relay_wire_protocol::{DeniedReason, UnavailableReason};

/// Why a locate request ended without a fix, with the agent-facing text and HTTP status.
pub enum LocateError {
    Offline {
        email: Option<String>,
    },
    /// The phone got the ping and is still connected, but sent no answer.
    Timeout,
    /// The phone got the ping, then every socket of the owner closed before an answer came.
    ConnectionLost,
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
    /// The requester asked this owner too often; holds the wait.
    RateLimited(Duration),
    /// The owner already has the maximum number of open requests.
    TooManyPending,
}

impl LocateError {
    pub fn status(&self) -> StatusCode {
        match self {
            Self::Offline { .. } => StatusCode::SERVICE_UNAVAILABLE,
            Self::Timeout | Self::ConnectionLost => StatusCode::GATEWAY_TIMEOUT,
            Self::Denied(DeniedReason::NoAnswer) => StatusCode::REQUEST_TIMEOUT,
            Self::Denied(_) | Self::DeniedBy(_) => StatusCode::FORBIDDEN,
            Self::Unavailable(_) => StatusCode::FAILED_DEPENDENCY,
            Self::NotShared(_) => StatusCode::NOT_FOUND,
            Self::Ambiguous { .. } => StatusCode::CONFLICT,
            Self::RateLimited(_) | Self::TooManyPending => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    /// Seconds for the `Retry-After` header of a 429.
    pub fn retry_after_secs(&self) -> Option<u64> {
        match self {
            Self::RateLimited(wait) => Some(wait.as_secs_f64().ceil().max(1.0) as u64),
            // An open request ends within its timeout; a short wait is a fair guess.
            Self::TooManyPending => Some(10),
            _ => None,
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
            Self::ConnectionLost => {
                "The phone got the request, then lost its connection before it could answer (weak or no network). Ask the user to check the phone's network, then retry.".into()
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
            Self::RateLimited(_) | Self::TooManyPending => format!(
                "Too many location requests. Wait {} seconds before asking again.",
                self.retry_after_secs().unwrap_or(10)
            ),
        }
    }
}
