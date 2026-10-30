//! Open locate requests: the waiters phones answer, and how many each owner has open.

use std::{
    collections::{HashMap, hash_map::Entry},
    sync::{MutexGuard, PoisonError},
};

use uuid::Uuid;

use crate::{AppState, location_request_error::LocateError, phone_message_handler::Waiter};

#[derive(Default)]
pub struct Pending {
    pub waiters: HashMap<Uuid, Waiter>,
    per_owner: HashMap<Uuid, usize>,
}

/// Locks the open requests. No caller holds the lock across an `.await`.
pub fn lock(state: &AppState) -> MutexGuard<'_, Pending> {
    state.pending.lock().unwrap_or_else(PoisonError::into_inner)
}

/// One open request. Dropping it frees the waiter and the owner's slot, also when the
/// caller's future is cancelled (client hung up), so an owner is never locked out.
pub struct PendingSlot<'a> {
    state: &'a AppState,
    id: Uuid,
    owner: Uuid,
}

impl<'a> PendingSlot<'a> {
    /// Registers `waiter` under `id`, unless its owner is at the cap or `requester` is over its rate.
    pub fn open(
        state: &'a AppState,
        requester: Uuid,
        id: Uuid,
        waiter: Waiter,
    ) -> Result<Self, LocateError> {
        let owner = waiter.0;
        // One lock for check and insert, so concurrent asks cannot pass the cap together.
        let mut pending = lock(state);
        if pending.per_owner.get(&owner).copied().unwrap_or(0) >= state.locate_limiter.max_pending {
            return Err(LocateError::TooManyPending);
        }
        state
            .locate_limiter
            .check(requester, owner)
            .map_err(LocateError::RateLimited)?;
        *pending.per_owner.entry(owner).or_default() += 1;
        pending.waiters.insert(id, waiter);
        Ok(Self { state, id, owner })
    }
}

impl Drop for PendingSlot<'_> {
    fn drop(&mut self) {
        let mut pending = lock(self.state);
        pending.waiters.remove(&self.id);
        if let Entry::Occupied(mut open) = pending.per_owner.entry(self.owner) {
            *open.get_mut() -= 1;
            if *open.get() == 0 {
                open.remove();
            }
        }
    }
}
