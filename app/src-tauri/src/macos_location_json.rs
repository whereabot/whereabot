//! CoreLocation values as the JSON the web app reads (tauri-plugin-geolocation's Position shape).

use objc2_core_location::{CLAuthorizationStatus, CLLocation};
use serde_json::{Value, json};

pub fn state(status: CLAuthorizationStatus) -> &'static str {
    match status {
        CLAuthorizationStatus::AuthorizedAlways | CLAuthorizationStatus::AuthorizedWhenInUse => "granted",
        CLAuthorizationStatus::NotDetermined => "prompt",
        _ => "denied",
    }
}

/// Same shape as tauri-plugin-geolocation's Position. Negative accuracy means "not valid".
pub fn position(l: &CLLocation) -> Value {
    let valid = |v: f64| if v < 0.0 { Value::Null } else { v.into() };
    unsafe {
        let c = l.coordinate();
        let vertical = l.verticalAccuracy();
        json!({
            "timestamp": (l.timestamp().timeIntervalSince1970() * 1000.0) as u64,
            "coords": {
                "latitude": c.latitude, "longitude": c.longitude, "accuracy": l.horizontalAccuracy(),
                "altitude": if vertical < 0.0 { Value::Null } else { l.altitude().into() },
                "altitudeAccuracy": valid(vertical), "heading": valid(l.course()), "speed": valid(l.speed()),
            }
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use objc2::AllocAnyThread;

    #[test]
    fn maps_status_and_fix() {
        assert_eq!(state(CLAuthorizationStatus::AuthorizedAlways), "granted");
        assert_eq!(state(CLAuthorizationStatus::NotDetermined), "prompt");
        assert_eq!(state(CLAuthorizationStatus::Restricted), "denied");
        let l = unsafe { CLLocation::initWithLatitude_longitude(CLLocation::alloc(), 52.52, 13.405) };
        let p = position(&l);
        assert_eq!(p["coords"]["latitude"], 52.52);
        assert_eq!(p["coords"]["longitude"], 13.405);
        // No vertical fix: altitude is null, not a made-up 0.
        assert!(p["coords"]["altitude"].is_null() && p["coords"]["altitudeAccuracy"].is_null());
    }
}
