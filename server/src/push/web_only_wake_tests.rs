//! A user with only the web app (or a stale native install) has nothing a silent push can wake.

use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

use uuid::Uuid;

use super::{fake_apns_and_postgrest::start, push_wake_tests::alert};

#[tokio::test]
async fn always_allowed_without_a_live_app_install_gets_the_alert_at_once() {
    // Web app only (2026-11-03): no install can answer a silent push, so the 8 s wait is lost time.
    let (push, fake) = start().await;
    fake.web_only.store(true, Ordering::Relaxed);
    let started = Instant::now();
    push.wake_after(
        Duration::ZERO,
        "jwt",
        &alert(Uuid::new_v4(), "trusted-agent"),
    )
    .await;
    assert!(started.elapsed() < push.silent_window);
    let pushes = fake.pushes.lock().unwrap();
    assert!(
        pushes
            .iter()
            .all(|(token, h, _)| h["apns-push-type"] == "alert" && token != "cc33")
    );
    assert_eq!(pushes[0].2["aps"]["category"], "LOCATION_AUTO");
}
