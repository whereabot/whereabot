use std::time::{Duration, SystemTime, UNIX_EPOCH};

use std::sync::Arc;

use tokio::sync::oneshot;
use uuid::Uuid;

use crate::{
    Agent, AppState,
    devices::location_request_fanout::Fanout,
    locate_outcome_log,
    locate_request::LocateRequest,
    location_request_error::LocateError,
    location_target::Target,
    pending_location_requests::PendingSlot,
    push::location_request_wake,
    relay_wire_protocol::{Answer, DeniedReason, Location},
};

/// Pings the target owner's phones for `agent` and waits for the fix.
pub async fn request_location(
    state: &AppState,
    agent: &Agent,
    target: Target,
    reason: Option<String>,
    timeout: Duration,
) -> Result<Location, LocateError> {
    let owner = target.owner;
    // iOS drops the socket in the background: wait for the app, and resend the
    // ping to each new socket so a reconnect does not lose it.
    let deadline = tokio::time::Instant::now() + timeout;
    let expires_ms = (SystemTime::now() + timeout)
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_millis() as u64);
    let request = Arc::new(LocateRequest {
        id: Uuid::new_v4(),
        client_id: agent.client_id.clone(),
        expires_ms,
        requester: target.requester,
        reason,
    });
    let (tx, mut rx) = oneshot::channel();
    // Keyed by the owner: only the owner's phones may answer. Dropped on every exit.
    let _slot = PendingSlot::open(state, agent.user_id, request.id, (owner, tx))?;
    let connected = !state.devices.senders(owner).await.is_empty();
    let shared_owner = request.requester.is_some().then_some(owner);
    let wake = location_request_wake::start(
        state.push.as_ref(),
        agent.bearer.as_deref(),
        connected,
        Arc::clone(&request),
        shared_owner,
    );
    let mut fanout = Fanout::new(owner, Arc::clone(&request));
    let mut winner = Uuid::nil();
    let result = loop {
        fanout.send_new(&state.devices).await;
        if tokio::time::Instant::now() >= deadline {
            break Err(if !fanout.reached() {
                LocateError::Offline {
                    email: target.email.clone(),
                }
            } else if state.devices.senders(owner).await.is_empty() {
                LocateError::ConnectionLost
            } else {
                LocateError::Timeout
            });
        }
        if let Ok(answer) = tokio::time::timeout(Duration::from_millis(250), &mut rx).await {
            break match answer {
                Ok((from, answer)) => {
                    winner = from;
                    match answer {
                        Answer::Fix(location) => Ok(location),
                        Answer::Denied(reason) => Err(match (reason, &target.name) {
                            (DeniedReason::User, Some(name)) => LocateError::DeniedBy(name.clone()),
                            _ => LocateError::Denied(reason),
                        }),
                        Answer::Unavailable(reason) => Err(LocateError::Unavailable(reason)),
                    }
                }
                Err(_) => Err(LocateError::Timeout),
            };
        }
    };
    drop(_slot);
    drop(wake);
    fanout.cancel_others(&state.devices, winner).await;
    locate_outcome_log::log(request.id, &result);
    result
}
