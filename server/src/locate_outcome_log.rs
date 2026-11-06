//! One log line per locate request with how it ended, so a support case can be traced.
//! Outcomes where the user wanted an answer and got none log at WARN: filter on
//! `severity>=WARNING` to find every failed locate.

use std::time::Duration;

use uuid::Uuid;

use crate::{location_request_error::LocateError, relay_wire_protocol::Location};

/// How request `id` ended, and how long it waited out of `wait`. `reached` is whether a
/// phone got the ping. The line never carries user data.
pub fn log(
    id: Uuid,
    result: &Result<Location, LocateError>,
    elapsed: Duration,
    wait: Duration,
    reached: bool,
) {
    let (outcome, failed) = outcome(result);
    let (elapsed_ms, wait_s) = (elapsed.as_millis() as u64, wait.as_secs());
    if failed {
        tracing::warn!(request_id = %id, outcome, elapsed_ms, wait_s, phone_reached = reached, "locate request failed");
    } else {
        tracing::info!(request_id = %id, outcome, elapsed_ms, wait_s, phone_reached = reached, "locate request ended");
    }
}

/// The outcome name, and whether the user got no answer for a reason we may need to fix.
/// A user's denial or an agent's own limits are not failures.
fn outcome(result: &Result<Location, LocateError>) -> (&'static str, bool) {
    match result {
        Ok(_) => ("fix", false),
        Err(LocateError::Offline { .. }) => ("offline", true),
        Err(LocateError::Timeout) => ("no_answer", true),
        Err(LocateError::ConnectionLost) => ("connection_lost", true),
        Err(LocateError::Unavailable(_)) => ("unavailable", true),
        Err(LocateError::Denied(_) | LocateError::DeniedBy(_)) => ("denied", false),
        Err(LocateError::NotShared(_)) => ("not_shared", false),
        Err(LocateError::Ambiguous { .. }) => ("ambiguous", false),
        Err(LocateError::RateLimited(_) | LocateError::TooManyPending) => ("rate_limited", false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missed_answers_are_failures_and_choices_are_not() {
        assert_eq!(outcome(&Err(LocateError::Timeout)), ("no_answer", true));
        assert_eq!(
            outcome(&Err(LocateError::ConnectionLost)),
            ("connection_lost", true)
        );
        assert_eq!(
            outcome(&Err(LocateError::TooManyPending)),
            ("rate_limited", false)
        );
    }
}
