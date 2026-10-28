#![allow(dead_code)]
//! Test signing key (ES256, made once per test binary), token builders, and the JWKS that publishes it.

use std::{
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode, jwk::Jwk};
use p256::{SecretKey, elliptic_curve::Generate, pkcs8::EncodePrivateKey};
use serde_json::{Value, json};

pub const KID: &str = "test-kid";

pub struct Key {
    encoding: EncodingKey,
    jwk: Jwk,
}

/// The ES256 key, generated on first use.
pub fn key() -> &'static Key {
    static KEY: OnceLock<Key> = OnceLock::new();
    KEY.get_or_init(|| {
        let der = SecretKey::generate().to_pkcs8_der().unwrap();
        let encoding = EncodingKey::from_ec_der(der.as_bytes());
        let mut jwk = Jwk::from_encoding_key(&encoding, Algorithm::ES256).unwrap();
        jwk.common.key_id = Some(KID.into());
        Key { encoding, jwk }
    })
}

/// The JWKS that Supabase would publish for this key.
pub fn jwks() -> Value {
    json!({ "keys": [serde_json::to_value(&key().jwk).unwrap()] })
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Claims of a valid Supabase access token. `client` adds the OAuth client id that MCP needs.
pub fn claims(issuer: &str, user: &str, client: bool) -> Value {
    json!({
        "iss": issuer,
        "aud": "authenticated",
        "sub": user,
        "role": "authenticated",
        "exp": now_secs() + 3600,
        "client_id": client.then_some("claude-code"),
    })
}

/// ES256 token with the test key and the default `kid`.
pub fn sign(claims: &Value) -> String {
    sign_with_kid(claims, KID)
}

pub fn sign_with_kid(claims: &Value, kid: &str) -> String {
    let mut header = Header::new(Algorithm::ES256);
    header.kid = Some(kid.into());
    encode(&header, claims, &key().encoding).unwrap()
}

/// HS256 token signed with a secret the relay never holds.
pub fn sign_hs256(claims: &Value) -> String {
    let mut header = Header::new(Algorithm::HS256);
    header.kid = Some(KID.into());
    encode(
        &header,
        claims,
        &EncodingKey::from_secret(b"not-the-jwks-key"),
    )
    .unwrap()
}

/// Unsigned token with `alg: none`.
pub fn sign_none(claims: &Value) -> String {
    let header = json!({ "alg": "none", "typ": "JWT", "kid": KID });
    let header = URL_SAFE_NO_PAD.encode(header.to_string());
    let body = URL_SAFE_NO_PAD.encode(claims.to_string());
    format!("{header}.{body}.")
}
