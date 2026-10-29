//! Test double for Web Push: the subscriptions (`fake_web_push_postgrest`), the grants list,
//! and a push service that answers 410 for the `gone` subscription.

use std::sync::{Arc, Mutex, OnceLock};

use axum::{
    Router,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD as B64};
use web_push_native::{jwt_simple::algorithms::ES256KeyPair, p256::SecretKey};

use crate::push::{
    Push, PushConfig,
    fake_loopback_server::{grants, serve},
};

pub struct FakeWeb {
    pub url: OnceLock<String>,
    /// The browser's key: tests decrypt what the relay sent with it.
    pub ua: SecretKey,
    /// (subscription name, headers, encrypted body) per push received.
    pub pushes: Mutex<Vec<(String, HeaderMap, Bytes)>>,
    pub deleted: Mutex<Vec<String>>,
}

pub type Shared = State<Arc<FakeWeb>>;

async fn push_service(
    State(f): Shared,
    Path(name): Path<String>,
    h: HeaderMap,
    b: Bytes,
) -> StatusCode {
    let gone = name == "gone";
    f.pushes.lock().unwrap().push((name, h, b));
    if gone {
        StatusCode::GONE
    } else {
        StatusCode::CREATED
    }
}

pub async fn start() -> (Push, Arc<FakeWeb>) {
    let fake = Arc::new(FakeWeb {
        url: OnceLock::new(),
        ua: SecretKey::from_slice(&[7; 32]).unwrap(),
        pushes: Mutex::default(),
        deleted: Mutex::default(),
    });
    let app = Router::new()
        .merge(super::fake_web_push_postgrest::routes())
        .route("/auth/v1/user/oauth/grants", get(grants))
        .route("/push/{name}", post(push_service))
        .with_state(fake.clone());
    let url = serve(app).await;
    fake.url.set(url.clone()).unwrap();
    let config = PushConfig {
        apns: None,
        vapid_private_key: Some(B64.encode(ES256KeyPair::generate().to_bytes())),
        supabase_url: url,
        anon_key: "anon".into(),
    };
    (Push::new(&config).unwrap(), fake)
}
