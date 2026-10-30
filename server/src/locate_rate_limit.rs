//! Anti-flood limits on locate requests: each one alerts the owner's phone.

use std::{num::NonZeroU32, time::Duration};

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
            std::env::var(key).ok().and_then(|v| v.parse().ok())
        }
        let default = Self::default();
        Self {
            per_minute: get("WB_LOCATE_PER_MINUTE").unwrap_or(default.per_minute),
            burst: get("WB_LOCATE_BURST").unwrap_or(default.burst),
            max_pending: get("WB_LOCATE_MAX_PENDING")
                .filter(|max| *max > 0)
                .unwrap_or(default.max_pending),
        }
    }
}

pub struct LocateLimiter {
    pairs: DefaultKeyedRateLimiter<(Uuid, Uuid)>,
    pub max_pending: usize,
}

impl LocateLimiter {
    pub fn new(limits: LocateLimits) -> Self {
        let quota = Quota::per_minute(limits.per_minute).allow_burst(limits.burst);
        Self {
            pairs: RateLimiter::keyed(quota),
            max_pending: limits.max_pending,
        }
    }

    /// `Err` holds the wait before `requester` may ask `owner` again.
    pub fn check(&self, requester: Uuid, owner: Uuid) -> Result<(), Duration> {
        // shortcut: prunes idle pairs only past 10k keys; move to a timed task if that gets hot.
        if self.pairs.len() > 10_000 {
            self.pairs.retain_recent();
        }
        self.pairs
            .check_key(&(requester, owner))
            .map_err(|not_until| not_until.wait_time_from(self.pairs.clock().now()))
    }
}
