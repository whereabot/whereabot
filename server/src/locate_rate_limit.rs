//! Anti-flood limits on locate requests: each one alerts the owner's phone.

use std::{
    num::{NonZeroU32, NonZeroUsize},
    sync::{Mutex, PoisonError},
    time::{Duration, Instant},
};

use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter, clock::Clock};
use uuid::Uuid;

/// Locate limits. Defaults sit far above normal use (a few asks an hour).
#[derive(Clone, Copy)]
pub struct LocateLimits {
    /// `WB_LOCATE_PER_MINUTE`: steady locates per requester→owner pair.
    pub per_minute: NonZeroU32,
    /// `WB_LOCATE_BURST`: locates a pair may send back to back.
    pub burst: NonZeroU32,
    /// `WB_LOCATE_MAX_PENDING`: open requests per owner, from all requesters.
    pub max_pending: usize,
}

impl Default for LocateLimits {
    fn default() -> Self {
        Self {
            per_minute: NonZeroU32::new(6).expect("nonzero"),
            burst: NonZeroU32::new(3).expect("nonzero"),
            max_pending: 3,
        }
    }
}

impl LocateLimits {
    /// Reads the `WB_LOCATE_*` variables; a missing or invalid one keeps its default.
    pub fn from_env() -> Self {
        fn get<T: std::str::FromStr>(key: &str) -> Option<T> {
            let value = std::env::var(key).ok()?;
            let parsed = value.parse().ok();
            if parsed.is_none() {
                tracing::warn!("{key}={value:?} is not a positive integer; using the default");
            }
            parsed
        }
        let default = Self::default();
        Self {
            per_minute: get("WB_LOCATE_PER_MINUTE").unwrap_or(default.per_minute),
            burst: get("WB_LOCATE_BURST").unwrap_or(default.burst),
            max_pending: get("WB_LOCATE_MAX_PENDING")
                .map_or(default.max_pending, NonZeroUsize::get),
        }
    }
}

pub struct LocateLimiter {
    pairs: DefaultKeyedRateLimiter<(Uuid, Uuid)>,
    last_prune: Mutex<Instant>,
    pub max_pending: usize,
}

/// How often idle pairs are dropped from the limiter's map.
const PRUNE_EVERY: Duration = Duration::from_secs(60);

impl LocateLimiter {
    pub fn new(limits: LocateLimits) -> Self {
        let quota = Quota::per_minute(limits.per_minute).allow_burst(limits.burst);
        Self {
            pairs: RateLimiter::keyed(quota),
            last_prune: Mutex::new(Instant::now()),
            max_pending: limits.max_pending,
        }
    }

    /// `Err` holds the wait before `requester` may ask `owner` again.
    pub fn check(&self, requester: Uuid, owner: Uuid) -> Result<(), Duration> {
        // O(keys) prune, at most once per PRUNE_EVERY, so the map stays bounded.
        let mut last = self
            .last_prune
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if last.elapsed() >= PRUNE_EVERY {
            *last = Instant::now();
            drop(last);
            self.pairs.retain_recent();
        }
        self.pairs
            .check_key(&(requester, owner))
            .map_err(|not_until| not_until.wait_time_from(self.pairs.clock().now()))
    }
}
