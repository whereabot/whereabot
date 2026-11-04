//! Which install a push row belongs to (`devices(revoked_at)` embed), shared by APNs and Web Push.

use serde::Deserialize;

/// The `devices(revoked_at)` embed on a push row.
#[derive(Deserialize)]
pub(in crate::push) struct DeviceRow {
    revoked_at: Option<String>,
}

/// Same rule as the share RPCs: an unlinked row counts, a revoked device's row does not.
pub(in crate::push) fn not_revoked(device: Option<&DeviceRow>) -> bool {
    device.is_none_or(|d| d.revoked_at.is_none())
}

/// One APNs token. `linked`: it belongs to a live app install, which a silent push can wake.
pub struct Token {
    pub token: String,
    pub linked: bool,
}
