//! Checks one Supabase access token (C3) against the key it names.

use jsonwebtoken::{Algorithm, AlgorithmFamily, DecodingKey, Validation, decode};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct Claims {
    sub: Uuid,
    client_id: Option<String>,
    email: Option<String>,
}

/// A token that passed every C3 rule.
pub struct Verified {
    pub user_id: Uuid,
    pub client_id: Option<String>,
    pub email: Option<String>,
}

/// Who the token names when every C3 rule passes. `client_id` is required for MCP, not for the phone.
pub fn check(
    token: &str,
    key: &DecodingKey,
    issuer: &str,
    require_client_id: bool,
) -> Option<Verified> {
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
    let client_id = claims.client_id.filter(|id| !id.is_empty());
    (!require_client_id || client_id.is_some()).then_some(Verified {
        user_id: claims.sub,
        client_id,
        email: claims.email.filter(|email| !email.is_empty()),
    })
}
