//! Subscription endpoints are user-written rows: the relay only POSTs to known browser push services.

use reqwest::Url;

/// Chrome/Edge (FCM), Firefox (Mozilla autopush), Safari (Apple), legacy Edge (WNS).
const PUSH_SERVICES: [&str; 4] = [
    "fcm.googleapis.com",
    "push.services.mozilla.com",
    "push.apple.com",
    "notify.windows.com",
];

/// True for https endpoints on a known push service host or one of its subdomains.
pub fn trusted(endpoint: &Url) -> bool {
    let host = endpoint.host_str().unwrap_or_default();
    if cfg!(test) && host == "127.0.0.1" {
        return true; // the fake push service in tests
    }
    endpoint.scheme() == "https"
        && PUSH_SERVICES
            .iter()
            .any(|s| host == *s || host.ends_with(&format!(".{s}")))
}

#[cfg(test)]
mod tests {
    use super::trusted;

    fn ok(url: &str) -> bool {
        trusted(&url.parse().unwrap())
    }

    #[test]
    fn only_https_push_services() {
        assert!(ok("https://fcm.googleapis.com/fcm/send/abc"));
        assert!(ok("https://updates.push.services.mozilla.com/wpush/v2/x"));
        assert!(ok("https://web.push.apple.com/QGx"));
        assert!(ok("https://wns2-db5p.notify.windows.com/w/?token=x"));
        assert!(!ok("http://fcm.googleapis.com/fcm/send/abc"));
        assert!(!ok("https://evil-fcm.googleapis.com.example/x"));
        assert!(!ok("https://metadata.google.internal/"));
        assert!(!ok("https://notpush.apple.com.evil/x"));
    }
}
