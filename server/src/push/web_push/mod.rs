//! Web Push for browser/PWA installs: subscriptions from Supabase, VAPID-signed encrypted pushes.

#[cfg(test)]
mod fake_web_push_postgrest;
#[cfg(test)]
mod fake_web_push_service;
mod web_push_endpoint_allowlist;
mod web_push_fanout;
#[cfg(test)]
mod web_push_fanout_tests;
mod web_push_sender;
mod web_push_subscription_store;

pub use web_push_sender::WebPush;
pub use web_push_subscription_store::Subscriptions;
