//! One log line per locate request with how it ended, so a support case can be traced.

use uuid::Uuid;

use crate::{location_request_error::LocateError, relay_wire_protocol::Location};

/// Logs the outcome of request `id`. The outcome name never carries user data.
pub fn log(id: Uuid, result: &Result<Location, LocateError>) {
    let outcome = match result {
        Ok(_) => "fix",
        Err(LocateError::Offline { .. }) => "offline",
        Err(LocateError::Timeout) => "no_answer",
        Err(LocateError::ConnectionLost) => "connection_lost",
        Err(LocateError::Denied(_) | LocateError::DeniedBy(_)) => "denied",
        Err(LocateError::Unavailable(_)) => "unavailable",
        Err(LocateError::NotShared(_)) => "not_shared",
        Err(LocateError::Ambiguous { .. }) => "ambiguous",
        Err(LocateError::RateLimited(_) | LocateError::TooManyPending) => "rate_limited",
    };
    tracing::info!(request_id = %id, outcome, "locate request ended");
}
