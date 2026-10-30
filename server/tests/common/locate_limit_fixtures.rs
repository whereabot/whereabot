#![allow(dead_code)]
//! Locate limits for test relays.

use std::num::NonZeroU32;

use whereabouts_server::LocateLimits;

/// Tests retry locates fast, so most relays get limits nothing reaches.
pub const LOOSE: LocateLimits = limits(10_000, 10_000, 100);

/// Limits with the given rate per minute, burst, and pending cap.
pub const fn limits(per_minute: u32, burst: u32, max_pending: usize) -> LocateLimits {
    LocateLimits {
        per_minute: NonZeroU32::new(per_minute).unwrap(),
        burst: NonZeroU32::new(burst).unwrap(),
        max_pending,
    }
}
