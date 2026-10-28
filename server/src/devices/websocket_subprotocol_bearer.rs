use axum::http::{
    HeaderMap, HeaderValue,
    header::{AUTHORIZATION, SEC_WEBSOCKET_PROTOCOL},
};

/// Browsers cannot set Authorization on a WebSocket. The browser app (PWA) offers the
/// subprotocols `whereabot.bearer, <token>` instead; the relay picks `whereabot.bearer`.
pub const BEARER_PROTOCOL: &str = "whereabot.bearer";

/// Moves the subprotocol token into Authorization when the header is absent, so the usual
/// bearer check covers both. A real Authorization header always wins.
pub fn with_subprotocol_bearer(mut headers: HeaderMap) -> HeaderMap {
    if headers.contains_key(AUTHORIZATION) {
        return headers;
    }
    let token = headers
        .get(SEC_WEBSOCKET_PROTOCOL)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            let mut parts = v.split(',').map(str::trim);
            parts.find(|p| *p == BEARER_PROTOCOL)?;
            parts.next().map(str::to_owned)
        });
    if let Some(value) = token.and_then(|t| HeaderValue::from_str(&format!("Bearer {t}")).ok()) {
        headers.insert(AUTHORIZATION, value);
    }
    headers
}
