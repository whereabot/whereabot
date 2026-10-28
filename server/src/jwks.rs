//! Supabase signing keys, cached. Refetched when stale, or on an unknown `kid` at most once a minute.
//! After a failed fetch, no fetch for 30 s; meanwhile the last good keys answer.

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use jsonwebtoken::{DecodingKey, jwk::JwkSet};
use tokio::sync::Mutex;

use crate::supabase::{call, http};

const TTL: Duration = Duration::from_secs(600);
const REFETCH_GAP: Duration = Duration::from_secs(60);
const FAILURE_GAP: Duration = Duration::from_secs(30);

pub struct JwksCache {
    url: String,
    http: reqwest::Client,
    // Held across the fetch, so concurrent misses share one request.
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    keys: HashMap<String, DecodingKey>,
    fetched_at: Option<Instant>,
    refetched_at: Option<Instant>,
    failed_at: Option<Instant>,
}

/// True when no fetch ran in the last `gap`.
fn due(at: Option<Instant>, gap: Duration) -> bool {
    at.is_none_or(|at| at.elapsed() >= gap)
}

impl JwksCache {
    pub fn new(supabase_url: &str) -> Self {
        Self {
            url: format!("{supabase_url}/auth/v1/.well-known/jwks.json"),
            http: http(),
            inner: Mutex::default(),
        }
    }

    /// Key for a token's `kid`. None when Supabase has no such key.
    pub async fn key(&self, kid: &str) -> Option<DecodingKey> {
        let mut inner = self.inner.lock().await;
        let stale = inner.fetched_at.is_none_or(|at| at.elapsed() > TTL);
        if stale && due(inner.failed_at, FAILURE_GAP) {
            self.load(&mut inner).await;
        }
        if let Some(key) = inner.keys.get(kid) {
            return Some(key.clone());
        }
        if !due(inner.refetched_at, REFETCH_GAP) || !due(inner.failed_at, FAILURE_GAP) {
            return None;
        }
        inner.refetched_at = Some(Instant::now());
        self.load(&mut inner).await;
        inner.keys.get(kid).cloned()
    }

    /// Replaces the keys. A failed fetch keeps the old keys and starts the 30 s failure gap.
    async fn load(&self, inner: &mut Inner) {
        let res = self.http.get(&self.url).send().await;
        let Some(set) = call::<JwkSet>("jwks", res).await else {
            inner.failed_at = Some(Instant::now());
            return;
        };
        inner.failed_at = None;
        inner.keys = set
            .keys
            .iter()
            .filter_map(|jwk| {
                let kid = jwk.common.key_id.clone()?;
                Some((kid, DecodingKey::from_jwk(jwk).ok()?))
            })
            .collect();
        inner.fetched_at = Some(Instant::now());
    }
}
