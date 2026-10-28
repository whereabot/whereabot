//! Checks one Supabase access token (C3) against the key it names.

use jsonwebtoken::{Algorithm, AlgorithmFamily, DecodingKey, Validation, decode};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct Claims {
    sub: Uuid,
    client_id: Option<String>,
}

/// User id when every C3 rule passes. `client_id` is required for MCP, not for the phone.
pub fn check(
    token: &str,
    key: &DecodingKey,
    issuer: &str,
    require_client_id: bool,
) -> Option<Uuid> {
    // The key decides the one allowed algorithm. `none` and HS256 never match an EC or RSA key.
    let alg = match key.family() {
        AlgorithmFamily::Ec => Algorithm::ES256,
        AlgorithmFamily::Rsa => Algorithm::RS256,
        _ => return None,
    };
    let mut validation = Validation::new(alg);
    validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
    validation.set_issuer(&[issuer]);
    validation.set_audience(&["authenticated"]);
    validation.leeway = 30;
    let claims = decode::<Claims>(token, key, &validation).ok()?.claims;
    let has_client = claims.client_id.is_some_and(|id| !id.is_empty());
    (!require_client_id || has_client).then_some(claims.sub)
}
